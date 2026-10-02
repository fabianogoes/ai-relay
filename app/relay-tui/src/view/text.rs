//! Text helpers for a grid of terminal columns: wrapping, truncating and the
//! relative time of a handoff. Pure: the clock is a parameter.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// `text` cut to at most `max` columns, ending in `…` when something was cut.
pub fn truncate(text: &str, max: usize) -> String {
    if width(text) <= max {
        return text.to_string();
    }
    if max == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > max - 1 {
            break;
        }
        out.push(c);
        used += w;
    }
    out.push('…');
    out
}

/// `text` broken into lines of at most `max` columns. Whitespace collapses; a
/// word wider than a line is broken where it overflows.
pub fn wrap(text: &str, max: usize) -> Vec<String> {
    if max == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for word in text.split_whitespace() {
        let mut word = word.to_string();
        // A word that cannot fit on a line of its own is cut to fit.
        while width(&word) > max {
            if used > 0 {
                lines.push(std::mem::take(&mut line));
                used = 0;
            }
            let mut head = String::new();
            let mut head_width = 0;
            let mut rest = String::new();
            for c in word.chars() {
                let w = c.width().unwrap_or(0);
                if rest.is_empty() && head_width + w <= max {
                    head.push(c);
                    head_width += w;
                } else {
                    rest.push(c);
                }
            }
            lines.push(head);
            word = rest;
        }
        let w = width(&word);
        if used == 0 {
            line.push_str(&word);
            used = w;
        } else if used + 1 + w <= max {
            line.push(' ');
            line.push_str(&word);
            used += 1 + w;
        } else {
            lines.push(std::mem::take(&mut line));
            line.push_str(&word);
            used = w;
        }
    }
    if used > 0 {
        lines.push(line);
    }
    lines
}

/// `wrap`, capped at `max_lines`; the last kept line ends in `…` when the text
/// went on.
pub fn wrap_capped(text: &str, max: usize, max_lines: usize) -> Vec<String> {
    let mut lines = wrap(text, max);
    if lines.len() > max_lines && max_lines > 0 {
        lines.truncate(max_lines);
        // `last …` fits when there is room; otherwise the cut itself ends in `…`.
        let last = lines.pop().unwrap();
        lines.push(truncate(&format!("{last} …"), max));
    }
    lines
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The Unix time of an RFC 3339 timestamp (`2026-10-02T09:00:00-03:00`), or
/// `None` when it does not have that shape.
pub fn parse_rfc3339(value: &str) -> Option<i64> {
    let b = value.as_bytes();
    if b.len() < 20 || !value.is_ascii() {
        return None;
    }
    let num = |from: usize, to: usize| value.get(from..to)?.parse::<i64>().ok();
    if b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let (year, month, day) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (hour, minute, second) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    let offset = match &value[19..] {
        "Z" => 0,
        tail if tail.len() == 6 && tail.as_bytes()[3] == b':' => {
            let sign = match tail.as_bytes()[0] {
                b'+' => 1,
                b'-' => -1,
                _ => return None,
            };
            sign * (tail.get(1..3)?.parse::<i64>().ok()? * 3600 + tail.get(4..6)?.parse::<i64>().ok()? * 60)
        }
        _ => return None,
    };
    let days = days_from_civil(year, month, day);
    Some(days * 86_400 + hour * 3600 + minute * 60 + second - offset)
}

/// How long ago `updated` was, in Portuguese; an unparseable value is returned
/// as it came. A timestamp in the future (a skewed clock) reads as `agora`.
pub fn relative_time(updated: &str, now_unix: i64) -> String {
    let Some(then) = parse_rfc3339(updated) else {
        return updated.to_string();
    };
    let seconds = now_unix - then;
    match seconds {
        s if s < 60 => "agora".to_string(),
        s if s < 3600 => format!("há {} min", s / 60),
        s if s < 86_400 => format!("há {} h", s / 3600),
        s => format!("há {} d", s / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_glyphs_of_the_design_are_one_column_wide() {
        for glyph in ["✓", "●", "○", "◌", "•", "…", "·", "╭", "─", "│"] {
            assert_eq!(width(glyph), 1, "{glyph}");
        }
    }

    #[test]
    fn truncate_keeps_what_fits_and_marks_the_cut() {
        assert_eq!(truncate("curto", 10), "curto");
        assert_eq!(truncate("exatamente", 10), "exatamente");
        assert_eq!(truncate("um texto bem comprido", 10), "um texto …");
        assert_eq!(truncate("abc", 1), "…");
        assert_eq!(truncate("abc", 0), "");
        // A wide character is never split: 日本語 is 6 columns.
        assert_eq!(truncate("日本語", 5), "日本…");
    }

    #[test]
    fn wrap_breaks_on_spaces_and_collapses_whitespace() {
        assert_eq!(wrap("um  dois\ntres quatro", 9), ["um dois", "tres", "quatro"]);
        assert_eq!(wrap("", 10), Vec::<String>::new());
        assert_eq!(wrap("cabe", 10), ["cabe"]);
    }

    #[test]
    fn wrap_breaks_a_word_wider_than_the_line() {
        assert_eq!(wrap("abcdefghij", 4), ["abcd", "efgh", "ij"]);
        assert_eq!(wrap("ab abcdefgh", 4), ["ab", "abcd", "efgh"]);
    }

    #[test]
    fn wrap_capped_ends_the_last_line_with_an_ellipsis() {
        assert_eq!(wrap_capped("um dois tres quatro cinco", 9, 2), ["um dois", "tres …"]);
        assert_eq!(wrap_capped("um dois", 20, 2), ["um dois"]);
        // A full last line still shows that the text went on.
        let lines = wrap_capped("aaaa bbbb cccc", 4, 2);
        assert_eq!(lines.len(), 2);
        assert!(lines[1].ends_with('…') && width(&lines[1]) <= 4, "{lines:?}");
    }

    #[test]
    fn rfc3339_gives_the_unix_time() {
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_rfc3339("2000-01-01T00:00:00Z"), Some(946_684_800));
        // The offset is applied: 09:00 at UTC-3 is 12:00 UTC.
        assert_eq!(
            parse_rfc3339("2026-10-02T09:00:00-03:00"),
            parse_rfc3339("2026-10-02T12:00:00Z")
        );
        assert_eq!(
            parse_rfc3339("2026-10-02T12:00:00+05:30"),
            parse_rfc3339("2026-10-02T06:30:00Z")
        );
        assert_eq!(parse_rfc3339("ontem"), None);
        assert_eq!(parse_rfc3339("2026-10-02 12:00:00Z"), None);
    }

    #[test]
    fn relative_time_reads_in_portuguese() {
        let then = parse_rfc3339("2026-10-02T12:00:00Z").unwrap();
        let at = |plus: i64| relative_time("2026-10-02T12:00:00Z", then + plus);
        assert_eq!(at(0), "agora");
        assert_eq!(at(59), "agora");
        assert_eq!(at(60), "há 1 min");
        assert_eq!(at(4 * 60 + 30), "há 4 min");
        assert_eq!(at(3 * 3600), "há 3 h");
        assert_eq!(at(86_400), "há 1 d");
        assert_eq!(at(3 * 86_400 + 5000), "há 3 d");
        // A skewed clock never says "há -2 min".
        assert_eq!(at(-120), "agora");
        assert_eq!(relative_time("ontem", then), "ontem");
    }
}
