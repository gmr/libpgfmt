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
    /// A quoted identifier, `"..."`, whose spacing is part of the name.
    Identifier,
}

/// The span starting at `chars[i]`, with the index just past its end.
///
/// Returns `None` when no literal or comment starts there. An unterminated
/// span ends at the end of the input.
pub(crate) fn scan(chars: &[char], i: usize) -> Option<(Span, usize)> {
    match chars.get(i)? {
        '\'' => Some((Span::Literal, quoted_end(chars, i + 1, false))),
        '"' => Some((Span::Identifier, identifier_end(chars, i + 1))),
        'E' | 'e' if chars.get(i + 1) == Some(&'\'') && !continues_identifier(chars, i) => {
            Some((Span::Literal, quoted_end(chars, i + 2, true)))
        }
        '$' if !continues_identifier(chars, i) => {
            dollar_end(chars, i).map(|end| (Span::Literal, end))
        }
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

/// Whether `chars[i]` continues the identifier before it, in which case no
/// literal can start there.
///
/// PostgreSQL allows `$` inside an unquoted identifier, and requires
/// whitespace between an identifier and a following dollar quote — so the `$`
/// in `foo$tag$` opens no literal. An `E` or `e` directly after an identifier
/// character is likewise part of that identifier, not an escape-string prefix.
fn continues_identifier(chars: &[char], i: usize) -> bool {
    i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_' || chars[i - 1] == '$')
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

/// End of a quoted identifier whose body starts at `i`. A doubled quote is an
/// escaped quote.
fn identifier_end(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() {
        match chars[i] {
            '"' if chars.get(i + 1) == Some(&'"') => i += 2,
            '"' => return i + 1,
            _ => i += 1,
        }
    }
    chars.len()
}

/// Collapse whitespace runs to a single space and trim, with two exceptions:
/// a run that contains a newline and sits between two string constants is
/// kept as a newline, and so is the newline that terminates a `--` comment.
/// String constants, quoted identifiers and comments are copied verbatim.
///
/// PostgreSQL concatenates two string constants only when a line break
/// separates them; on one line the adjacency is a syntax error. Collapsing
/// that newline turns `SELECT 'foo'\n'bar'` into invalid SQL. Collapsing the
/// newline after a `--` comment is worse: the comment then swallows the rest
/// of the statement.
pub(crate) fn collapse_whitespace(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut after_line_comment = false;
    while i < chars.len() {
        let c = chars[i];
        if let Some((span, end)) = scan(&chars, i) {
            out.extend(&chars[i..end]);
            after_line_comment = span == Span::LineComment;
            i = end;
            continue;
        }
        if c.is_whitespace() {
            let mut j = i;
            let mut saw_newline = false;
            while j < chars.len() && chars[j].is_whitespace() {
                saw_newline |= chars[j] == '\n';
                j += 1;
            }
            let between_literals =
                saw_newline && out.ends_with('\'') && j < chars.len() && chars[j] == '\'';
            if !out.is_empty() && j < chars.len() {
                out.push(if between_literals || (saw_newline && after_line_comment) {
                    '\n'
                } else {
                    ' '
                });
            }
            after_line_comment = false;
            i = j;
            continue;
        }
        out.push(c);
        after_line_comment = false;
        i += 1;
    }
    out
}

/// Whether `text` ends inside a `--` comment, so that anything appended on
/// the same line would be commented out.
pub(crate) fn ends_in_line_comment(text: &str) -> bool {
    let chars: Vec<char> = text.trim_end().chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if let Some((span, end)) = scan(&chars, i) {
            if span == Span::LineComment && end >= chars.len() {
                return true;
            }
            i = end;
        } else {
            i += 1;
        }
    }
    false
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
    fn quoted_identifiers() {
        assert_eq!(end_of("\"a  b\" x"), Some((Span::Identifier, 6)));
        assert_eq!(end_of("\"a\"\"b\" x"), Some((Span::Identifier, 6)));
    }

    #[test]
    fn collapse_keeps_meaningful_newlines() {
        assert_eq!(collapse_whitespace("  a   b  "), "a b");
        assert_eq!(collapse_whitespace("f(a, -- why\n  b)"), "f(a, -- why\nb)");
        assert_eq!(collapse_whitespace("'foo'\n   'bar'"), "'foo'\n'bar'");
        assert_eq!(collapse_whitespace("'a  b' \"c  d\""), "'a  b' \"c  d\"");
    }

    #[test]
    fn line_comment_at_end() {
        assert!(ends_in_line_comment("SELECT 1 -- why\n"));
        assert!(!ends_in_line_comment("SELECT '--' AS x"));
        assert!(!ends_in_line_comment("-- a\nSELECT 1"));
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
    fn no_literal_starts_inside_an_identifier() {
        // `$` and `E` are both legal inside an unquoted identifier.
        for (text, i) in [
            ("foo$tag$ + 1\nFROM bar$tag$", 3),
            ("foo$$tag$ + 1\nFROM bar$$tag$", 4),
            (r"ae'a\'b' + 1", 1),
        ] {
            let chars: Vec<char> = text.chars().collect();
            assert_eq!(scan(&chars, i), None, "{text} at {i}");
        }
    }

    #[test]
    fn comments() {
        assert_eq!(end_of("-- hi\nx"), Some((Span::LineComment, 5)));
        assert_eq!(end_of("/* a\nb */x"), Some((Span::BlockComment, 9)));
    }
}
