use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::cli::{Config, resolve_use_color};
use crate::fs_walk::collect_files;
use crate::regex::parse_regex;
use crate::search::process_input;

pub fn run(cfg: Config) -> i32 {
    let use_color = resolve_use_color(&cfg.color);

    let pattern_for_parser = if cfg.anchored {
        &cfg.pattern[1..]
    } else {
        &cfg.pattern[..]
    };
    let tokens = parse_regex(pattern_for_parser);

    let mut global_matched = false;

    if cfg.paths.is_empty() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        process_input(
            &buffer,
            &tokens,
            None,
            cfg.use_o,
            use_color,
            &mut global_matched,
            cfg.anchored,
            false,
        );
        return if global_matched { 0 } else { 1 };
    }

    // expand input paths to concrete files
    let mut files = Vec::new();
    for p in &cfg.paths {
        files.extend(collect_files(Path::new(p), cfg.recursive));
    }

    // mimic your old behavior: recursive always shows prefix; otherwise only when multiple files
    let show_filename = cfg.recursive || files.len() > 1;

    for path in files {
        if let Ok(content) = fs::read_to_string(&path) {
            let name = path.to_string_lossy();
            process_input(
                &content,
                &tokens,
                Some(name.as_ref()),
                cfg.use_o,
                use_color,
                &mut global_matched,
                cfg.anchored,
                show_filename,
            );
        }
    }

    if global_matched { 0 } else { 1 }
}
