use std::process;

use clap::Parser;

use disk_usage_clone::cli::CliArgs;
use disk_usage_clone::run;

fn main() {
    let args = CliArgs::parse();

    if let Err(err) = run(&args) {
        eprintln!("dusk: {err}");
        process::exit(1);
    }
}
