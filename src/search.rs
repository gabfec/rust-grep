use crate::output::maybe_colorize;
use crate::regex::{Token, match_pattern};

pub fn process_input(
    content: &str,
    tokens: &[Token],
    filename: Option<&str>,
    use_o: bool,
    use_color: bool,
    global_matched: &mut bool,
    is_anchored: bool,
    show_filename: bool,
) {
    let prefix = if show_filename {
        filename.map(|f| format!("{f}:")).unwrap_or_default()
    } else {
        String::new()
    };

    for line in content.lines() {
        let mut current_search_text = line;
        let mut line_buffer = String::new();
        let mut line_has_match = false;
        let mut last_match_end_in_line = 0;

        loop {
            if let Some(matched_slice) = match_pattern(current_search_text, tokens) {
                *global_matched = true;
                line_has_match = true;

                let match_text = maybe_colorize(matched_slice, use_color);

                if use_o {
                    println!("{prefix}{match_text}");
                } else {
                    let offset_in_line = line.len() - current_search_text.len();
                    line_buffer.push_str(&line[last_match_end_in_line..offset_in_line]);
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
                if is_anchored {
                    break;
                }
                let mut chars = current_search_text.chars();
                if chars.next().is_some() {
                    current_search_text = chars.as_str();
                } else {
                    break;
                }
            }
        }

        if !use_o && line_has_match {
            line_buffer.push_str(&line[last_match_end_in_line..]);
            println!("{prefix}{line_buffer}");
        }
    }
}
