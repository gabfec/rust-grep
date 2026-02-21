use crate::regex::ast::{GroupType, Token};

pub fn parse_regex(pattern: &str) -> Vec<Token> {
    let mut group_counter = 0;
    parse_pattern(pattern, &mut group_counter)
}

fn parse_pattern(pattern: &str, group_counter: &mut usize) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('d') => tokens.push(Token::Digit),
                Some('w') => tokens.push(Token::Alphanumeric),
                Some(digit) if digit.is_digit(10) => {
                    // Handle \1, \2, \3...
                    let n = digit.to_digit(10).unwrap() as usize;
                    tokens.push(Token::Backreference(n));
                }
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
                    if next_c == ']' {
                        break;
                    }
                    class_chars.push(next_c);
                }
                tokens.push(Token::BracketGroup(class_chars, group_type));
            }
            '(' => {
                *group_counter += 1;
                let current_group_id = *group_counter;

                // Collect everything inside the parentheses into a buffer
                let mut group_buffer = String::new();
                let mut depth = 1;

                while let Some(inner_c) = chars.next() {
                    if inner_c == '(' {
                        depth += 1;
                    } else if inner_c == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    group_buffer.push(inner_c);
                }

                // Split by '|' only at the top level of this group
                let mut parts = Vec::new();
                let mut current_part = String::new();
                let mut paren_depth = 0;
                for char in group_buffer.chars() {
                    if char == '(' {
                        paren_depth += 1;
                    } else if char == ')' {
                        paren_depth -= 1;
                    }

                    if char == '|' && paren_depth == 0 {
                        parts.push(current_part.clone());
                        current_part.clear();
                    } else {
                        current_part.push(char);
                    }
                }
                parts.push(current_part);

                if parts.len() > 1 {
                    let mut alt_token = Token::Alternation(
                        parse_pattern(&parts[0], group_counter),
                        parse_pattern(&parts[1], group_counter),
                    );

                    // Nest any additional parts
                    for part in parts.iter().skip(2) {
                        alt_token =
                            Token::Alternation(vec![alt_token], parse_pattern(part, group_counter));
                    }
                    tokens.push(Token::Group(vec![alt_token], current_group_id));
                } else {
                    // If no pipe, wrap the sequence in a Group
                    // This allows the next quantifier to pop the whole group
                    let group_tokens = parse_pattern(&group_buffer, group_counter);
                    tokens.push(Token::Group(group_tokens, current_group_id));
                }
            }
            '{' => {
                let mut buffer = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c == '}' {
                        chars.next();
                        break;
                    }
                    buffer.push(chars.next().unwrap());
                }
                if let Some(prev) = tokens.pop() {
                    let parts: Vec<&str> = buffer.split(',').collect();
                    let n = parts[0].trim().parse().unwrap_or(0);
                    if buffer.contains(',') {
                        let m = parts[1].trim().parse().ok();
                        tokens.push(Token::Quantifier(Box::new(prev), n, m));
                    } else {
                        tokens.push(Token::Quantifier(Box::new(prev), n, Some(n)));
                    }
                }
            }
            '+' => {
                if let Some(prev) = tokens.pop() {
                    tokens.push(Token::Quantifier(Box::new(prev), 1, None));
                }
            }
            '?' => {
                if let Some(prev) = tokens.pop() {
                    tokens.push(Token::Quantifier(Box::new(prev), 0, Some(1)));
                }
            }
            '*' => {
                if let Some(prev) = tokens.pop() {
                    tokens.push(Token::Quantifier(Box::new(prev), 0, None));
                }
            }
            '.' => tokens.push(Token::Wildcard),
            _ => tokens.push(Token::Literal(c)),
        }
    }
    tokens
}
