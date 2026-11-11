//! Log files written on threads of their own.
//!
//! A line for the log comes from the thread that logs it, often the one that owns the
//! windows. While Windows writes out a large file it has copied, every other write to that
//! disk waits, however small (36 seconds for one log line on a test PC with a 6 GB copy), and
//! the fences stopped for as long. [`background`] hands each line to a thread that does the
//! waiting. What must be in the file before the process goes on (the last line of an exit, a
//! panic, a crash, the end of the session) calls [`written`] after it.

use std::fs::File;
use std::io::{self, Write};
use std::sync::Mutex;
use std::sync::mpsc::{Sender, channel};
use std::time::{Duration, Instant};

/// How long the end of the process (an exit, a panic, a crash) waits for its last lines to
/// be written: the next start reads them to tell how this run ended. The disk can keep a
/// write waiting most of a minute while Windows writes out a large file.
pub const EXIT_WAIT: Duration = Duration::from_secs(90);

/// How long the thread that owns the windows waits for the lines so far where it could go
/// on without them: at the end of the session, which Windows cuts short anyway, and before
/// the log is read for a feedback report.
pub const LINE_WAIT: Duration = Duration::from_secs(5);

enum Msg {
    Bytes(Vec<u8>),
    /// Answered once everything sent before it is written.
    Written(Sender<()>),
}

/// The writer threads' channels, for [`written`].
static WRITERS: Mutex<Vec<Sender<Msg>>> = Mutex::new(Vec::new());

/// A file whose `write` hands the bytes to its writer thread and returns.
pub struct LogFile {
    tx: Sender<Msg>,
}

/// Starts a thread that writes `file`. Without a thread to be had, the lines are dropped.
pub fn background(file: File) -> LogFile {
    let (tx, rx) = channel();
    let spawned = std::thread::Builder::new()
        .name("pecofence-log".into())
        .spawn(move || {
            let mut file = file;
            for msg in rx {
                match msg {
                    Msg::Bytes(bytes) => {
                        let _ = file.write_all(&bytes);
                    }
                    Msg::Written(done) => {
                        let _ = done.send(());
                    }
                }
            }
        });
    if spawned.is_ok() {
        WRITERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(tx.clone());
    }
    LogFile { tx }
}

impl LogFile {
    /// Waits until every line handed to this file so far is written, `timeout` at most.
    /// False when the time ran out first (the disk is busy).
    pub fn written(&self, timeout: Duration) -> bool {
        all_written(std::slice::from_ref(&self.tx), timeout)
    }
}

impl Write for LogFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let _ = self.tx.send(Msg::Bytes(buf.to_vec()));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// [`LogFile::written`] for every file started with [`background`].
pub fn written(timeout: Duration) -> bool {
    let writers = WRITERS.lock().unwrap_or_else(|e| e.into_inner()).clone();
    all_written(&writers, timeout)
}

fn all_written(writers: &[Sender<Msg>], timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let (done, answers) = channel();
    let asked = writers
        .iter()
        .filter(|tx| tx.send(Msg::Written(done.clone())).is_ok())
        .count();
    (0..asked).all(|_| {
        answers
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("pecofence-log-test-{name}-{}", std::process::id()))
    }

    // The tests wait for their own file: the files of the other tests in this process are
    // in the same list, and one of them stands for a busy disk.

    #[test]
    fn lines_reach_the_file_in_order_once_written_returns() {
        let path = temp_file("order");
        let mut log = background(File::create(&path).unwrap());
        for i in 0..500 {
            writeln!(log, "line {i}").unwrap();
        }
        assert!(log.written(Duration::from_secs(10)));
        let text = std::fs::read_to_string(&path).unwrap();
        let expected: String = (0..500).map(|i| format!("line {i}\n")).collect();
        assert_eq!(text, expected);
    }

    #[test]
    fn lines_from_several_threads_stay_whole() {
        let path = temp_file("threads");
        let log = Mutex::new(background(File::create(&path).unwrap()));
        std::thread::scope(|scope| {
            for t in 0..4 {
                let log = &log;
                scope.spawn(move || {
                    for i in 0..200 {
                        let line = format!("thread {t} line {i}\n");
                        log.lock().unwrap().write_all(line.as_bytes()).unwrap();
                    }
                });
            }
        });
        assert!(log.lock().unwrap().written(Duration::from_secs(10)));
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 800);
        for t in 0..4 {
            let own: Vec<&str> = text
                .lines()
                .filter(|l| l.starts_with(&format!("thread {t} ")))
                .collect();
            let expected: Vec<String> = (0..200).map(|i| format!("thread {t} line {i}")).collect();
            assert_eq!(own, expected);
        }
    }

    #[test]
    fn a_write_returns_while_the_file_cannot_be_written() {
        // A pipe nobody reads stands for the busy disk: its writer waits once the pipe is full.
        #[cfg(unix)]
        {
            let (reader, writer) = std::io::pipe().unwrap();
            let mut log = background(File::from(std::os::fd::OwnedFd::from(writer)));
            let started = Instant::now();
            let block = vec![b'x'; 64 * 1024];
            for _ in 0..64 {
                log.write_all(&block).unwrap();
            }
            assert!(started.elapsed() < Duration::from_secs(2));
            assert!(!log.written(Duration::from_millis(300)));
            // Every file, this one among them: the time runs out as well.
            assert!(!written(Duration::from_millis(300)));
            // Read to the end on another thread: the writer thread gets through.
            let drain = std::thread::spawn(move || {
                let mut reader = reader;
                let mut sink = [0u8; 64 * 1024];
                let mut total = 0usize;
                while total < 64 * 64 * 1024 {
                    total += io::Read::read(&mut reader, &mut sink).unwrap();
                }
                total
            });
            assert!(log.written(Duration::from_secs(20)));
            assert_eq!(drain.join().unwrap(), 64 * 64 * 1024);
        }
    }
}
