use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() == 1 {
        input_line.contains(pattern)
    } else if pattern.eq("\\d") {
        return input_line.chars().filter(|x| x.is_ascii_digit()).count() > 0;
    } else if pattern.eq("\\w") {
        return input_line.chars().filter(|x| x.is_alphanumeric() || *x == '_').count() > 0;
    } else if pattern.starts_with('[') && pattern.ends_with(']') {
        return pattern.chars().skip(1).take(pattern.len() - 2).any(|x| input_line.contains(x));
    }
    else {
        panic!("Unhandled pattern: {}", pattern)
    }
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
