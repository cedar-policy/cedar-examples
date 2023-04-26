#![forbid(unsafe_code)]
// This program is exported as a binary named cedar-github-model-app

use cedar_github_model_app::{benchmark, CedarExitCode, Cli, Commands};

use clap::Parser;

fn main() -> CedarExitCode {
    match Cli::parse().command {
        Commands::Benchmark(args) => benchmark(&args),
    }
}
