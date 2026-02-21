const COLOR_START: &str = "\x1b[01;31m";
const COLOR_RESET: &str = "\x1b[m";

pub fn maybe_colorize(s: &str, use_color: bool) -> String {
    if use_color {
        format!("{COLOR_START}{s}{COLOR_RESET}")
    } else {
        s.to_string()
    }
}
