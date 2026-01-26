use clap::Parser;
use less_clone::cli::CliArgs;
use std::process;

fn main() {
    let args = CliArgs::parse();

    if let Err(err) = less_clone::run(&args) {
        eprintln!("{err}");
        process::exit(1);
    }
}
