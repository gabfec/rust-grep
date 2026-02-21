use crate::regex::ast::{GroupType, Token};

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
        }
        _ => false, // This covers EndAnchor and any other future positional tokens
    }
}

// Checks if the pattern matches starting exactly at the beginning of 'text'
fn match_here(tokens: &[Token], text: &str, captures: &mut Vec<Option<String>>) -> Option<usize> {
    if tokens.is_empty() {
        return Some(0); // Pattern exhausted, we matched!
    }

    match &tokens[0] {
        Token::EndAnchor => {
            if text.is_empty() {
                Some(0)
            } else {
                None
            }
        }
        Token::Alternation(left, right) => {
            // We need to find the best match at this position.
            // Standard engines usually pick the first branch that results in a successful
            // match for the whole pattern.

            // Try Left branch + rest
            let mut left_captures = captures.clone();
            if let Some(left_len) = match_here(left, text, &mut left_captures) {
                if let Some(rest_len) =
                    match_here(&tokens[1..], &text[left_len..], &mut left_captures)
                {
                    *captures = left_captures;
                    return Some(left_len + rest_len);
                }
            }
            let mut right_captures = captures.clone();
            if let Some(right_len) = match_here(right, text, &mut right_captures) {
                if let Some(rest_len) =
                    match_here(&tokens[1..], &text[right_len..], &mut right_captures)
                {
                    *captures = right_captures;
                    return Some(right_len + rest_len);
                }
            }
            None
        }
        Token::Group(inner_tokens, id) => {
            // Ensure the Vec is big enough to hold this group ID
            if captures.len() < *id {
                captures.resize(*id, None);
            }

            // Standard engines try to match as much as possible, then backtrack.
            for try_len in (0..=text.len()).rev() {
                let mut inner_caps = captures.clone();

                if let Some(group_len) = match_here(inner_tokens, &text[..try_len], &mut inner_caps)
                {
                    // The inner match must consume exactly the length we are testing
                    if group_len == try_len {
                        inner_caps[*id - 1] = Some(text[..group_len].to_string());

                        if let Some(rest_len) =
                            match_here(&tokens[1..], &text[group_len..], &mut inner_caps)
                        {
                            *captures = inner_caps;
                            return Some(group_len + rest_len);
                        }
                    }
                }
            }
            None
        }
        Token::Backreference(n) => {
            // Check if we have a capture for this index
            if let Some(Some(captured_val)) = captures.get(*n - 1) {
                if text.starts_with(captured_val.as_str()) {
                    let len = captured_val.len();
                    return match_here(&tokens[1..], &text[len..], captures)
                        .map(|rest_len| len + rest_len);
                }
            }
            None
        }
        Token::Quantifier(inner, min, max) => {
            // If we've hit the maximum allowed matches (Some(0)), move to the rest of the pattern
            if let Some(0) = max {
                return match_here(&tokens[1..], text, captures);
            }

            // Save captures state before greedy attempt
            let saved_captures = captures.clone();

            // Greedy Attempt: Try to match the 'inner' token once
            if let Some(inner_len) = match_here(&[*inner.clone()], text, captures) {
                // Only recurse if we actually consumed something OR we are satisfying 'min'
                if inner_len > 0 || *min > 0 {
                    let next_min = if *min > 0 { min - 1 } else { 0 };
                    let next_max = max.map(|m| m - 1);

                    // Construct the "next" state for the quantifier
                    let next_token = Token::Quantifier(inner.clone(), next_min, next_max);
                    let mut sequence = vec![next_token];
                    sequence.extend_from_slice(&tokens[1..]);

                    // Try to match as many as possible (Greedy)
                    if let Some(total_len) = match_here(&sequence, &text[inner_len..], captures) {
                        return Some(inner_len + total_len);
                    }
                }
            }

            // Backtracking/Fallback: Restore captures and try without matching this iteration
            *captures = saved_captures;
            if *min == 0 {
                match_here(&tokens[1..], text, captures)
            } else {
                None
            }
        }
        // Handle normal single-character tokens
        _ => {
            let mut text_chars = text.chars();
            if let Some(c) = text_chars.next() {
                if matches_token(&tokens[0], c) {
                    let char_len = c.len_utf8();
                    return match_here(&tokens[1..], &text[char_len..], captures)
                        .map(|rest_len| char_len + rest_len);
                }
            }
            None
        }
    }
}

pub fn match_pattern<'a>(input_line: &'a str, tokens: &[Token]) -> Option<&'a str> {
    let mut captures: Vec<Option<String>> = Vec::new();
    match_here(tokens, input_line, &mut captures).map(|len| &input_line[..len])
}


#[cfg(test)]
mod tests {
    use crate::regex::{match_pattern, parse_regex};

    fn m(pattern: &str, text: &str) -> Option<String> {
        let tokens = parse_regex(pattern);
        match_pattern(text, &tokens).map(|s| s.to_string())
    }

    #[test]
    fn matches_simple_prefix() {
        assert_eq!(m("abc", "abcdef"), Some("abc".into()));
        assert_eq!(m("abc", "ab"), None);
    }

    #[test]
    fn matches_wildcard() {
        assert_eq!(m("a.c", "abc"), Some("abc".into()));
        assert_eq!(m("a.c", "axc"), Some("axc".into()));
        assert_eq!(m("a.c", "ac"), None);
    }

    #[test]
    fn matches_digit_and_word_class() {
        assert_eq!(m(r"\d\d", "42xx"), Some("42".into()));
        assert_eq!(m(r"\d\d", "4axx"), None);

        assert_eq!(m(r"\w\w", "a_"), Some("a_".into()));
        assert_eq!(m(r"\w\w", "a-"), None);
    }

    #[test]
    fn matches_bracket_group_positive_and_negative() {
        assert_eq!(m("[abc]", "a"), Some("a".into()));
        assert_eq!(m("[abc]", "z"), None);

        assert_eq!(m("[^abc]", "z"), Some("z".into()));
        assert_eq!(m("[^abc]", "a"), None);
    }

    #[test]
    fn matches_end_anchor() {
        assert_eq!(m("abc$", "abc"), Some("abc".into()));
        assert_eq!(m("abc$", "abcd"), None);
    }

    #[test]
    fn matches_question_mark() {
        assert_eq!(m("ab?c", "abc"), Some("abc".into()));
        assert_eq!(m("ab?c", "ac"), Some("ac".into()));
        assert_eq!(m("ab?c", "abbc"), None);
    }

    #[test]
    fn matches_star() {
        assert_eq!(m("ab*c", "ac"), Some("ac".into()));
        assert_eq!(m("ab*c", "abc"), Some("abc".into()));
        assert_eq!(m("ab*c", "abbbc"), Some("abbbc".into()));
    }

    #[test]
    fn matches_plus() {
        assert_eq!(m("ab+c", "ac"), None);
        assert_eq!(m("ab+c", "abc"), Some("abc".into()));
        assert_eq!(m("ab+c", "abbbc"), Some("abbbc".into()));
    }

    #[test]
    fn matches_braced_quantifiers() {
        assert_eq!(m("a{3}", "aaab"), Some("aaa".into()));
        assert_eq!(m("a{3}", "aab"), None);

        assert_eq!(m("a{2,4}", "aaaaa"), Some("aaaa".into())); // greedy
        assert_eq!(m("a{2,}", "aaaaa"), Some("aaaaa".into())); // greedy to end
    }

    #[test]
    fn matches_group_and_backreference() {
        assert_eq!(m(r"(ab)\1", "abab"), Some("abab".into()));
        assert_eq!(m(r"(ab)\1", "abac"), None);

        assert_eq!(m(r"(\w\w)\1", "xyxy"), Some("xyxy".into()));
        assert_eq!(m(r"(\w\w)\1", "xyxz"), None);
    }

    #[test]
    fn matches_alternation_inside_group() {
        assert_eq!(m("(a|bc)d", "ad"), Some("ad".into()));
        assert_eq!(m("(a|bc)d", "bcd"), Some("bcd".into()));
        assert_eq!(m("(a|bc)d", "abcd"), None);
    }

    #[test]
    fn greedy_then_backtracks_when_needed() {
        // This checks your quantifier backtracking behavior.
        // a* should not “eat” the 'b' needed to match the rest.
        assert_eq!(m("a*ab", "aaab"), Some("aaab".into()));
    }

    #[test]
    fn group_quantifier_matches_whole_group() {
        // Ensure quantifier applies to Group (since parser wraps (...) as Group token)
        assert_eq!(m("(ab)+", "ababx"), Some("abab".into()));
        assert_eq!(m("(ab)+", "abx"), Some("ab".into()));
        assert_eq!(m("(ab)+", "ax"), None);
    }
}