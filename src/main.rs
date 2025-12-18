use std::env;
use std::io;
use std::io::Read;
use std::process;
use std::fs;


#[derive(Debug, Clone)]
enum GroupType {
    Positive, // [abc]
    Negative, // [^abc]
}

#[derive(Debug, Clone)]
enum Token {
    Literal(char),
    Digit,
    Alphanumeric,
    Wildcard,
    BracketGroup(Vec<char>, GroupType),
    EndAnchor,             // $
    OneOrMore(Box<Token>), // +
    ZeroOrOne(Box<Token>), // ?
    Alternation(Vec<Token>, Vec<Token>), // |
}

fn parse_pattern(pattern: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('d') => tokens.push(Token::Digit),
                Some('w') => tokens.push(Token::Alphanumeric),
                Some(escaped) => tokens.push(Token::Literal(escaped)),
                None => {}
            },
            '$' => tokens.push(Token::EndAnchor),
            '[' => {
                let mut group_type = GroupType::Positive;
                if chars.peek() == Some(&'^') {
                    group_type = GroupType::Negative;
                    chars.next();
                }
                let mut class_chars = Vec::new();
                while let Some(next_c) = chars.next() {
                    if next_c == ']' { break; }
                    class_chars.push(next_c);
                }
                tokens.push(Token::BracketGroup(class_chars, group_type));
            },
            '(' => {
                // 1. Collect everything inside the parentheses into a buffer
                let mut group_buffer = String::new();
                let mut depth = 1;

                while let Some(inner_c) = chars.next() {
                    if inner_c == '(' { depth += 1; }
                    else if inner_c == ')' {
                        depth -= 1;
                        if depth == 0 { break; }
                    }
                    group_buffer.push(inner_c);
                }

                // Split by '|' and create a nested tree of Alternations
                let parts: Vec<&str> = group_buffer.split('|').collect();
                let mut alt_token = parse_pattern(parts[0]);

                for part in parts.iter().skip(1) {
                    let next_branch = parse_pattern(part);
                    alt_token = vec![Token::Alternation(alt_token, next_branch)];
                }
                tokens.extend(alt_token);
            }
            '+' => {
                if let Some(prev) = tokens.pop() {
                    tokens.push(Token::OneOrMore(Box::new(prev)));
                }
            },
            '?' => {
                if let Some(prev) = tokens.pop() {
                    tokens.push(Token::ZeroOrOne(Box::new(prev)));
                }
            },
            '.' => tokens.push(Token::Wildcard),
            _ => tokens.push(Token::Literal(c)),
        }
    }
    tokens
}

fn matches_token(token: &Token, c: char) -> bool {
    match token {
        Token::Wildcard => true,
        Token::Literal(l) => c == *l,
        Token::Digit => c.is_ascii_digit(),
        Token::Alphanumeric => c.is_ascii_alphanumeric() || c == '_',
        Token::BracketGroup(members, group_type) => {
            let found = members.contains(&c);
            match group_type {
                GroupType::Positive => found,
                GroupType::Negative => !found,
            }
        },
        _ => false, // This covers EndAnchor and any other future positional tokens
    }
}

// Checks if the pattern matches starting exactly at the beginning of 'text'
fn match_here(tokens: &[Token], text: &str) -> Option<usize> {
    if tokens.is_empty() {
        return Some(0); // Pattern exhausted, we matched!
    }

    match &tokens[0] {
        Token::EndAnchor => { if text.is_empty() { Some(0) } else { None } }
        Token::Alternation(left, right) => {
            // We need to find the best match at this position.
            // Standard engines usually pick the first branch that results in a successful
            // match for the whole pattern.

            // Try Left branch + rest
            if let Some(left_len) = match_here(left, text) {
                if let Some(rest_len) = match_here(&tokens[1..], &text[left_len..]) {
                    return Some(left_len + rest_len);
                }
            }

            // Try Right branch + rest
            if let Some(right_len) = match_here(right, text) {
                if let Some(rest_len) = match_here(&tokens[1..], &text[right_len..]) {
                    return Some(right_len + rest_len);
                }
            }
            None
        }
        Token::ZeroOrOne(inner) => {
            // 1. Try the "One" case first (Greedy)
            let mut text_chars = text.chars();
            if let Some(c) = text_chars.next() {
                if matches_token(inner, c) {
                    // If the char matches, see if the REST of the pattern matches
                    if let Some(rest_len) = match_here(&tokens[1..], text_chars.as_str()) {
                        return Some(1 + rest_len);
                    }
                }
            }

            // 2. If the "One" case failed, try the "Zero" case (Skip)
            match_here(&tokens[1..], text)
        }
        Token::OneOrMore(inner) => {
            let mut text_chars = text.chars();
            match text_chars.next() {
                Some(c) if matches_token(inner, c) => {
                    // Path A: Match more of the same (stay on OneOrMore)
                    if let Some(len) = match_here(tokens, text_chars.as_str()) {
                        return Some(1 + len);
                    }
                    // Path B: Move to the next token
                    if let Some(len) = match_here(&tokens[1..], text_chars.as_str()) {
                        return Some(1 + len);
                    }
                    None
                }
                _ => None,
            }
        }
        // Handle normal single-character tokens
        _ => {
            let mut text_chars = text.chars();
            match text_chars.next() {
                Some(c) if matches_token(&tokens[0], c) => {
                    match_here(&tokens[1..], text_chars.as_str()).map(|len| 1 + len)
                }
                _ => None,
            }
        }
    }
}

fn match_pattern<'a>(input_line: &'a str, tokens: &[Token]) -> Option<&'a str> {
    match_here(tokens, input_line).map(|len| &input_line[..len])
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    let args: Vec<String> = env::args().collect();
    let use_o = args.contains(&"-o".to_string());

    // Find the pattern index
    let pattern_idx = args.iter().position(|r| r == "-E").expect("Missing -E") + 1;
    let pattern_str = &args[pattern_idx];

    // Collect all file paths (everything after the pattern)
    let file_paths = &args[pattern_idx + 1..];

    // Determine if we should prefix with filenames
    let show_filename = file_paths.len() > 1;

    let tokens = if pattern_str.starts_with('^') {
        parse_pattern(&pattern_str[1..])
    } else {
        parse_pattern(pattern_str)
    };

    let mut global_matched = false;

    // Handle Stdin if no files are provided
    if file_paths.is_empty() {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap();
        process_input(&buffer, &tokens, None, use_o, &mut global_matched, pattern_str.starts_with('^'), false);
    } else {
        // Loop through each file
        for path in file_paths {
            if let Ok(content) = fs::read_to_string(path) {
                process_input(&content, &tokens, Some(path), use_o, &mut global_matched, pattern_str.starts_with('^'), show_filename);
            }
        }
    }

    process::exit(if global_matched { 0 } else { 1 });
}

// Helper to handle the matching logic for a block of text (file or stdin)
fn process_input(
    content: &str,
    tokens: &[Token],
    filename: Option<&String>,
    use_o: bool,
    global_matched: &mut bool,
    is_anchored: bool,
    show_filename: bool
) {
    for line in content.lines() {
        let mut current_search_text = line;

        loop {
            if let Some(matched_slice) = match_pattern(current_search_text, tokens) {
                *global_matched = true;

                // Build the prefix (e.g., "fruits.txt:")
                let prefix = if show_filename && filename.is_some() {
                    format!("{}:", filename.unwrap())
                } else {
                    "".to_string()
                };

                if use_o {
                    println!("{}{}", prefix, matched_slice);
                } else {
                    println!("{}{}", prefix, line);
                    break; // Move to next line
                }

                let advance_by = if matched_slice.is_empty() { 1 } else { matched_slice.len() };
                if advance_by > current_search_text.len() { break; }
                current_search_text = &current_search_text[advance_by..];
                if is_anchored || current_search_text.is_empty() { break; }
            } else {
                if is_anchored || current_search_text.is_empty() { break; }
                let mut chars = current_search_text.chars();
                chars.next();
                current_search_text = chars.as_str();
            }
        }
    }
}
