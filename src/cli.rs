use std::io;
use std::io::IsTerminal;

#[derive(Debug, Clone)]
pub enum ColorWhen {
    Always,
    Never,
    Auto,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub pattern: String,
    pub anchored: bool,
    pub use_o: bool,
    pub recursive: bool,
    pub color: ColorWhen,
    pub paths: Vec<String>,
}

pub fn parse_args(args: Vec<String>) -> Config {
    let use_o = args.iter().any(|a| a == "-o");
    let recursive = args.iter().any(|a| a == "-r");

    let color = if args.iter().any(|a| a == "--color=always") {
        ColorWhen::Always
    } else if args.iter().any(|a| a == "--color=never") {
        ColorWhen::Never
    } else {
        // default grep-ish behavior: never unless asked; you can keep Auto if you want
        if args.iter().any(|a| a == "--color=auto") {
            ColorWhen::Auto
        } else {
            ColorWhen::Never
        }
    };

    let pattern_idx = args.iter().position(|r| r == "-E").expect("Missing -E") + 1;
    let pattern = args[pattern_idx].clone();
    let anchored = pattern.starts_with('^');

    let paths = args[pattern_idx + 1..].to_vec();

    Config {
        pattern,
        anchored,
        use_o,
        recursive,
        color,
        paths,
    }
}

pub fn resolve_use_color(color: &ColorWhen) -> bool {
    match color {
        ColorWhen::Always => true,
        ColorWhen::Never => false,
        ColorWhen::Auto => io::stdout().is_terminal(),
    }
}
