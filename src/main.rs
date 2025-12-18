use std::env;
use std::io;
use std::process;


//#[derive(Debug)]
enum GroupType {
    Positive, // [abc]
    Negative, // [^abc]
}

//#[derive(Debug)]
enum Token {
    Literal(char),
    Digit,
    Alphanumeric,
    Wildcard,
    BracketGroup(Vec<char>, GroupType),
    EndAnchor,             // $
    OneOrMore(Box<Token>), // +
    ZeroOrOne(Box<Token>), // ?
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
fn match_here(tokens: &[Token], text: &str) -> bool {
    if tokens.is_empty() {
        return true; // Pattern exhausted, we matched!
    }

    match &tokens[0] {
        Token::EndAnchor => return text.is_empty(),
        Token::ZeroOrOne(inner) => {
            // Path A: The "Zero" case (Skip this token entirely)
            // We check if the rest of the tokens match the current text.
            if match_here(&tokens[1..], text) {
                return true;
            }

            // Path B: The "One" case (Try to match the token once)
            let mut text_chars = text.chars();
            match text_chars.next() {
                Some(c) if matches_token(inner, c) => {
                    // If it matches, we move to the next token and the rest of the text
                    match_here(&tokens[1..], text_chars.as_str())
                }
                _ => false,
            }
        }
        Token::OneOrMore(inner) => {
            let mut text_chars = text.chars();
            match text_chars.next() {
                Some(c) => {
                    if matches_token(inner, c) {
                        // Path A: Match more of the same (stay on OneOrMore)
                        // Path B: Move to the next token
                        return match_here(tokens, text_chars.as_str()) || match_here(&tokens[1..], text_chars.as_str());
                    }
                    false
                }
                None => false, // Text ended before pattern
            }
        }
        // Handle normal single-character tokens
        _ => {
            let mut chars = text.chars();
            match chars.next() {
                Some(c) if matches_token(&tokens[0], c) => {
                    match_here(&tokens[1..], chars.as_str())
                }
                _ => false,
            }
        }
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let tokens = parse_pattern(pattern);

    // Check empty string case (important for some tests)
    if input_line.is_empty() && tokens.is_empty() {
        return true;
    }

    if pattern.starts_with('^') {
        // We slice from 1 to remove the '^'
        let tokens = parse_pattern(&pattern[1..]);
        return match_here(&tokens, input_line);
    }

    // .char_indices() gives us (byte_index, character)
    // We only care about the byte_index to slice correctly
    for (i, _) in input_line.char_indices() {
        if match_here(&tokens, &input_line[i..]) {
            return true;
        }
    }

    false
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
