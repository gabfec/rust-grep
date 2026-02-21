# Rust Grep Clone with Custom Regex Engine

[![Tests](https://github.com/gabfec/rust-grep/actions/workflows/rust.yml/badge.svg)](https://github.com/gabfec/rust-grep/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)]()
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-red.svg)]()
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

A `grep`-like CLI tool written in Rust, featuring a **custom-built regex engine** implemented from scratch.

This project does NOT use Rust's built-in `regex` crate. Instead, it implements:

- Regex parser
- AST representation
- Backtracking matcher
- Capture groups and backreferences
- Quantifiers (`*`, `+`, `?`, `{n}`, `{n,m}`, `{n,}`)
- Alternation (`|`)
- Character classes (`[abc]`, `[^abc]`)
- Anchors (`^`, `$`)
- CLI compatible with basic `grep` usage

---

## Features

### Regex engine

Supported syntax:

| Feature | Example |
|--------|--------|
Literal | `abc`
Wildcard | `.`
Digit class | `\d`
Word class | `\w`
Character class | `[abc]`
Negative class | `[^abc]`
Quantifiers | `a*`, `a+`, `a?`, `a{3}`, `a{2,5}`
Grouping | `(abc)`
Alternation | `(a|b)`
Backreference | `(ab)\1`
End anchor | `$`
Start anchor | `^`

---

### CLI options

| Option | Description |
|------|-------------|
`-E pattern` | regex pattern (required)
`-o` | print only matches
`-r` | recursive search
`--color=always` | force color
`--color=never` | disable color
`--color=auto` | color if terminal

---

## Example usage

Search stdin:

```bash
echo "hello123" | cargo run -- -E "\d+"
```

## Educational Purpose

This project was built as part of the CodeCrafters challenge, which focuses on implementing real-world systems from scratch.

The goal was to understand how regex engines work internally, including parsing, AST construction, and backtracking match algorithms, without relying on external regex libraries.

https://codecrafters.io

## License

MIT License — see LICENSE file for details.
