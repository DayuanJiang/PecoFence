//! `pecofence-cli`: drives a running PecoFence over its named pipe and prints JSON.
//!
//! Console subsystem on purpose (the GUI exe cannot reliably write to a terminal). Exit codes:
//! 0 ok, 1 the app returned an error, 2 usage, 3 not running, 4 timeout.

mod cli;
mod client;
mod describe;
mod local;
mod output;
mod run;

use std::io::IsTerminal;

use clap::Parser;
use pecofence_core::brand;

fn main() {
    let cli = cli::Cli::parse();
    let pretty = if cli.pretty {
        true
    } else if cli.compact {
        false
    } else {
        std::io::stdout().is_terminal()
    };
    let instance = cli
        .instance
        .or_else(|| brand::var("PECOFENCE_INSTANCE").ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let ctx = run::Ctx {
        instance,
        timeout_ms: cli.timeout,
        pretty,
    };
    let is_describe_catalog = matches!(cli.command, cli::Command::Describe { schema: None });
    let code = match run::run(&ctx, cli.command) {
        Ok(reply) if is_describe_catalog && pretty => {
            output::print_text(&describe::render_pretty(&reply.payload()), true);
            0
        }
        Ok(reply) => output::print_result(reply, pretty),
        Err(error) => output::print_error(&error, pretty),
    };
    std::process::exit(code);
}
