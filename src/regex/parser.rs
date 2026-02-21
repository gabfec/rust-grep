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


#[cfg(test)]
mod tests {
    use super::parse_regex;
    use crate::regex::ast::{GroupType, Token};

    #[test]
    fn parses_literals() {
        let t = parse_regex("abc");
        assert_eq!(
            t,
            vec![Token::Literal('a'), Token::Literal('b'), Token::Literal('c')]
        );
    }

    #[test]
    fn parses_escapes_digit_and_word_and_literal_escape() {
        let t = parse_regex(r"\d\w\.");
        assert_eq!(
            t,
            vec![Token::Digit, Token::Alphanumeric, Token::Literal('.')]
        );
    }

    #[test]
    fn parses_wildcard_and_end_anchor() {
        let t = parse_regex("a.$");
        assert_eq!(
            t,
            vec![Token::Literal('a'), Token::Wildcard, Token::EndAnchor]
        );
    }

    #[test]
    fn parses_bracket_group_positive() {
        let t = parse_regex("[abc]");
        assert_eq!(
            t,
            vec![Token::BracketGroup(vec!['a', 'b', 'c'], GroupType::Positive)]
        );
    }

    #[test]
    fn parses_bracket_group_negative() {
        let t = parse_regex("[^abc]");
        assert_eq!(
            t,
            vec![Token::BracketGroup(vec!['a', 'b', 'c'], GroupType::Negative)]
        );
    }

    #[test]
    fn parses_quantifiers_question_star_plus() {
        let t = parse_regex("a?b*c+");
        assert_eq!(
            t,
            vec![
                Token::Quantifier(Box::new(Token::Literal('a')), 0, Some(1)),
                Token::Quantifier(Box::new(Token::Literal('b')), 0, None),
                Token::Quantifier(Box::new(Token::Literal('c')), 1, None),
            ]
        );
    }

    #[test]
    fn parses_braced_quantifier_exact() {
        let t = parse_regex("a{3}");
        assert_eq!(
            t,
            vec![Token::Quantifier(Box::new(Token::Literal('a')), 3, Some(3))]
        );
    }

    #[test]
    fn parses_braced_quantifier_min_only() {
        let t = parse_regex("a{2,}");
        assert_eq!(
            t,
            vec![Token::Quantifier(Box::new(Token::Literal('a')), 2, None)]
        );
    }

    #[test]
    fn parses_braced_quantifier_range() {
        let t = parse_regex("a{2,4}");
        assert_eq!(
            t,
            vec![Token::Quantifier(Box::new(Token::Literal('a')), 2, Some(4))]
        );
    }

    #[test]
    fn parses_group_assigns_id_1() {
        let t = parse_regex("(ab)");
        assert_eq!(
            t,
            vec![Token::Group(vec![Token::Literal('a'), Token::Literal('b')], 1)]
        );
    }

    #[test]
    fn parses_nested_groups_increment_ids() {
        let t = parse_regex("(a(b))");
        // Outer group gets id=1, inner group gets id=2 (based on your group_counter behavior)
        assert_eq!(
            t,
            vec![Token::Group(
                vec![
                    Token::Literal('a'),
                    Token::Group(vec![Token::Literal('b')], 2)
                ],
                1
            )]
        );
    }

    #[test]
    fn parses_alternation_inside_group() {
        let t = parse_regex("(a|bc)");
        assert_eq!(
            t,
            vec![Token::Group(
                vec![Token::Alternation(
                    vec![Token::Literal('a')],
                    vec![Token::Literal('b'), Token::Literal('c')]
                )],
                1
            )]
        );
    }

    #[test]
    fn parses_backreference() {
        let t = parse_regex(r"(ab)\1");
        assert_eq!(
            t,
            vec![
                Token::Group(vec![Token::Literal('a'), Token::Literal('b')], 1),
                Token::Backreference(1)
            ]
        );
    }

    #[test]
    fn parses_three_way_alternation_nesting() {
        // The parser nests alternations for more than 2 parts.
        let t = parse_regex("(a|b|c)");

        // Expected nesting:
        // Alternation( Alternation(a,b), c ) wrapped in Group(id=1)
        assert_eq!(
            t,
            vec![Token::Group(
                vec![Token::Alternation(
                    vec![Token::Alternation(
                        vec![Token::Literal('a')],
                        vec![Token::Literal('b')]
                    )],
                    vec![Token::Literal('c')]
                )],
                1
            )]
        );
    }
}