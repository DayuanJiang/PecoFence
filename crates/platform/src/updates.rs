//! Run the embedded Windows updater without a console or a dependency on PATH.

use std::fs::{self, File};
use std::io::Write;
use std::os::windows::fs::MetadataExt;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};

const WORKER: &str = include_str!("../../../scripts/runtime/update-worker.ps1");

/// Refuse redirects through directory junctions and symlinks before the app itself
/// creates any update files. The worker repeats this check before every operation.
pub fn plain_path(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_attributes() & 0x400 != 0 => {
                return Err(format!(
                    "Update paths cannot contain links or junctions: {}",
                    ancestor.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("{}: {error}", ancestor.display())),
        }
    }
    Ok(())
}

/// Persist an update plan before handing it to a worker. Never truncate the last
/// valid plan: recovery must survive interruption while a new parent PID is saved.
pub fn write_plan(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporary = path.with_extension("json.tmp");
    plain_path(path)?;
    plain_path(&temporary)?;
    let result = (|| -> std::io::Result<()> {
        let mut file = File::create(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    result.map_err(|e| format!("{}: {e}", path.display()))
}

pub fn start(stage: &Path, operation: &str) -> Result<Child, String> {
    if !matches!(operation, "Check" | "Download" | "Apply" | "Recover") {
        return Err("Unknown updater operation".into());
    }
    plain_path(stage)?;
    let worker = stage.join("update-worker.ps1");
    plain_path(&worker)?;
    // Always use the script embedded in this binary, including recovery after restart.
    write_plan(&worker, WORKER.as_bytes())?;
    let windows = std::env::var_os("SystemRoot").ok_or("SystemRoot is unavailable")?;
    let powershell = Path::new(&windows).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let log = stage.join(format!("{operation}.log"));
    plain_path(&log)?;
    let stderr = File::create(log).map_err(|e| e.to_string())?;
    Command::new(powershell)
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(worker)
        .args(["-Action", operation, "-Plan"])
        .arg(stage.join("plan.json"))
        .arg("-ParentPid")
        .arg(std::process::id().to_string())
        .current_dir(stage)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr)
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| format!("Could not start the Windows update worker: {e}"))
}
