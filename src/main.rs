mod app;
mod cli;
mod fs_walk;
mod output;
mod regex;
mod search;

use std::env;
use std::process;

fn main() {
    let cfg = cli::parse_args(env::args().collect());
    process::exit(app::run(cfg));
}
