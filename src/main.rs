mod regex;

use std::env;
use std::fs;
use std::io;
use std::io::{IsTerminal, Read};
use std::path::Path;
use std::process;

use crate::regex::Token;
use crate::regex::match_pattern;
use crate::regex::parse_regex;

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    let args: Vec<String> = env::args().collect();
    let use_o = args.contains(&"-o".to_string());
    let recursive = args.contains(&"-r".to_string());

    let use_color = if args.contains(&"--color=always".to_string()) {
        true
    } else if args.contains(&"--color=never".to_string()) {
        false
    } else if args.contains(&"--color=auto".to_string()) {
        io::stdout().is_terminal()
    } else {
        // Default grep behavior is usually 'never' unless specified
        false
    };

    // Find the pattern index
    let pattern_idx = args.iter().position(|r| r == "-E").expect("Missing -E") + 1;
    let pattern_str = &args[pattern_idx];

    // Collect all file paths (everything after the pattern)
    let file_paths = &args[pattern_idx + 1..];

    let tokens = if pattern_str.starts_with('^') {
        parse_regex(&pattern_str[1..])
    } else {
        parse_regex(pattern_str)
    };

    let mut global_matched = false;

    // Handle Stdin if no files are provided
    if file_paths.is_empty() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        process_input(
            &buffer,
            &tokens,
            None,
            use_o,
            use_color,
            &mut global_matched,
            pattern_str.starts_with('^'),
            false,
        );
    } else {
        // Loop through each file
        for path_str in file_paths {
            let path = Path::new(path_str);
            if recursive && path.is_dir() {
                // Recursive mode: always show prefix
                visit_dirs(
                    path,
                    &tokens,
                    use_o,
                    use_color,
                    &mut global_matched,
                    pattern_str.starts_with('^'),
                );
            } else if path.is_file() {
                // If multiple files were passed at the CLI, show filename
                let show_filename = file_paths.len() > 1;
                if let Ok(content) = fs::read_to_string(path) {
                    process_input(
                        &content,
                        &tokens,
                        Some(&path_str.to_string()),
                        use_o,
                        use_color,
                        &mut global_matched,
                        pattern_str.starts_with('^'),
                        show_filename,
                    );
                }
            }
        }
    }

    process::exit(if global_matched { 0 } else { 1 });
}

// Recursive function to walk directories
fn visit_dirs(
    dir: &Path,
    tokens: &[Token],
    use_o: bool,
    use_color: bool,
    global_matched: &mut bool,
    is_anchored: bool,
) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, tokens, use_o, use_color, global_matched, is_anchored);
            } else if let Ok(content) = fs::read_to_string(&path) {
                let path_display = path.to_string_lossy().to_string();
                process_input(
                    &content,
                    tokens,
                    Some(&path_display),
                    use_o,
                    use_color,
                    global_matched,
                    is_anchored,
                    true,
                );
            }
        }
    }
}

const COLOR_START: &str = "\x1b[01;31m";
const COLOR_RESET: &str = "\x1b[m";

// Helper to handle the matching logic for a block of text (file or stdin)
fn process_input(
    content: &str,
    tokens: &[Token],
    filename: Option<&String>,
    use_o: bool,
    use_color: bool,
    global_matched: &mut bool,
    is_anchored: bool,
    show_filename: bool,
) {
    let prefix = if show_filename && filename.is_some() {
        format!("{}:", filename.unwrap())
    } else {
        "".to_string()
    };

    for line in content.lines() {
        let mut current_search_text = line;
        let mut line_buffer = String::new(); // Used to reconstruct the line for multiple highlights
        let mut line_has_match = false;
        let mut last_match_end_in_line = 0;

        loop {
            if let Some(matched_slice) = match_pattern(current_search_text, tokens) {
                *global_matched = true;
                line_has_match = true;

                let match_text = if use_color {
                    format!("{COLOR_START}{matched_slice}{COLOR_RESET}")
                } else {
                    matched_slice.to_string()
                };

                if use_o {
                    // -o mode: print each match immediately
                    println!("{prefix}{match_text}");
                } else {
                    // Standard mode: calculate where we are in the original line
                    let offset_in_line = line.len() - current_search_text.len();

                    // Add the "gap" between the last match and this one
                    line_buffer.push_str(&line[last_match_end_in_line..offset_in_line]);
                    // Add the colored match
                    line_buffer.push_str(&match_text);

                    last_match_end_in_line = offset_in_line + matched_slice.len();
                }

                if is_anchored {
                    break;
                }

                let advance_by = if matched_slice.is_empty() {
                    1
                } else {
                    matched_slice.len()
                };
                if advance_by > current_search_text.len() {
                    break;
                }
                current_search_text = &current_search_text[advance_by..];
            } else {
                // No match at current index. Slide the window 1 character to try matching at index 1, 2, etc.
                if is_anchored {
                    break;
                }

                let mut chars = current_search_text.chars();
                if let Some(_) = chars.next() {
                    current_search_text = chars.as_str();
                } else {
                    break;
                }
            }
        }

        // After searching the whole line, if we matched in standard mode, print the reconstructed line
        if !use_o && line_has_match {
            // Append the rest of the line after the last match
            line_buffer.push_str(&line[last_match_end_in_line..]);
            println!("{prefix}{line_buffer}");
        }
    }
}
