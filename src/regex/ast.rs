#[derive(Debug, Clone)]
pub enum GroupType {
    Positive, // [abc]
    Negative, // [^abc]
}

#[derive(Debug, Clone)]
pub enum Token {
    Literal(char),
    Digit,
    Alphanumeric,
    Wildcard,
    BracketGroup(Vec<char>, GroupType),
    EndAnchor,                                    // $
    Quantifier(Box<Token>, usize, Option<usize>), // {n,}, {n,}, {n,m}, ?, *, +
    Alternation(Vec<Token>, Vec<Token>),          // |
    Group(Vec<Token>, usize),                     // Index of this group
    Backreference(usize),                         // \1, \2, etc.
}
