//! A small PostgreSQL-aware scanner over formatted SQL text.
//!
//! Several formatting passes rewrite whitespace by scanning text rather than
//! the CST. They must not touch the inside of a string literal, whose spacing
//! is data, nor the inside of a comment, whose terminating newline is
//! load-bearing. This module gives those passes one shared notion of where
//! such a span starts and ends.

/// What a scanned span is, so a caller can treat literals and comments
/// differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Span {
    /// A string constant: `'...'`, `E'...'` or `$tag$...$tag$`.
    Literal,
    /// A `-- ...` comment, ending before its terminating newline.
    LineComment,
    /// A `/* ... */` comment.
    BlockComment,
}

/// The span starting at `chars[i]`, with the index just past its end.
///
/// Returns `None` when no literal or comment starts there. An unterminated
/// span ends at the end of the input.
pub(crate) fn scan(chars: &[char], i: usize) -> Option<(Span, usize)> {
    match chars.get(i)? {
        '\'' => Some((Span::Literal, quoted_end(chars, i + 1, false))),
        'E' | 'e' if chars.get(i + 1) == Some(&'\'') => {
            Some((Span::Literal, quoted_end(chars, i + 2, true)))
        }
        '$' => dollar_end(chars, i).map(|end| (Span::Literal, end)),
        '-' if chars.get(i + 1) == Some(&'-') => {
            let end = chars[i..]
                .iter()
                .position(|c| *c == '\n')
                .map_or(chars.len(), |p| i + p);
            Some((Span::LineComment, end))
        }
        '/' if chars.get(i + 1) == Some(&'*') => {
            let mut j = i + 2;
            while j + 1 < chars.len() && !(chars[j] == '*' && chars[j + 1] == '/') {
                j += 1;
            }
            Some((Span::BlockComment, chars.len().min(j + 2)))
        }
        _ => None,
    }
}

/// End of a single-quoted constant whose body starts at `i`. A doubled quote
/// is an escaped quote in every form; a backslash escapes the next character
/// only in an `E'...'` string.
fn quoted_end(chars: &[char], mut i: usize, escapes: bool) -> usize {
    while i < chars.len() {
        match chars[i] {
            '\\' if escapes => i += 2,
            '\'' if chars.get(i + 1) == Some(&'\'') => i += 2,
            '\'' => return i + 1,
            _ => i += 1,
        }
    }
    chars.len()
}

/// End of a dollar-quoted constant starting at `i`, or `None` when the `$` is
/// something else — a positional parameter, say.
fn dollar_end(chars: &[char], i: usize) -> Option<usize> {
    let mut j = i + 1;
    while chars
        .get(j)
        .is_some_and(|c| c.is_alphanumeric() || *c == '_')
    {
        j += 1;
    }
    if chars.get(j) != Some(&'$') || chars.get(i + 1).is_some_and(char::is_ascii_digit) {
        return None;
    }
    let tag = &chars[i..=j];
    let mut k = j + 1;
    while k + tag.len() <= chars.len() {
        if &chars[k..k + tag.len()] == tag {
            return Some(k + tag.len());
        }
        k += 1;
    }
    Some(chars.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn end_of(text: &str) -> Option<(Span, usize)> {
        let chars: Vec<char> = text.chars().collect();
        scan(&chars, 0)
    }

    #[test]
    fn plain_and_doubled_quotes() {
        assert_eq!(end_of("'ab' x"), Some((Span::Literal, 4)));
        assert_eq!(end_of("'a''b' x"), Some((Span::Literal, 6)));
    }

    #[test]
    fn backslash_escapes_only_in_escape_strings() {
        assert_eq!(end_of(r"E'a\'b' x"), Some((Span::Literal, 7)));
        assert_eq!(end_of(r"'a\' x"), Some((Span::Literal, 4)));
    }

    #[test]
    fn dollar_quotes() {
        assert_eq!(end_of("$$a b$$ x"), Some((Span::Literal, 7)));
        assert_eq!(end_of("$tag$a$$b$tag$ x"), Some((Span::Literal, 14)));
        assert_eq!(end_of("$1 + 2"), None);
    }

    #[test]
    fn comments() {
        assert_eq!(end_of("-- hi\nx"), Some((Span::LineComment, 5)));
        assert_eq!(end_of("/* a\nb */x"), Some((Span::BlockComment, 9)));
    }
}
