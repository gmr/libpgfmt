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

/// Put back the original spelling of every string constant and quoted
/// identifier in `output` whose text differs from one in `source` only in
/// whitespace.
///
/// Layout code indents formatted text line by line, and a line that
/// continues a multi-line literal is indented with the rest, which changes
/// the literal's value -- and changes it again on every pass. Rather than
/// teach each of those sites about literals, this runs once over the result.
/// Literals are matched by content, not position, so it holds when the
/// formatter reorders clauses; one it changed on purpose, beyond whitespace,
/// matches nothing and is left alone. Dollar-quoted bodies are excluded:
/// their re-indentation is deliberate and decided where the body is
/// rendered. See https://github.com/gmr/libpgfmt/issues/57.
pub(crate) fn restore_literals(source: &str, output: &str) -> String {
    let key = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    let mut originals: Vec<String> = quoted_spans(source)
        .into_iter()
        .map(|(start, end)| source.chars().skip(start).take(end - start).collect())
        .collect();
    let chars: Vec<char> = output.chars().collect();
    let spans = quoted_spans(output);
    let texts: Vec<String> = spans
        .iter()
        .map(|&(start, end)| chars[start..end].iter().collect())
        .collect();

    // Unchanged literals claim their original first, so a changed one cannot
    // take the spelling that belongs to an identical literal elsewhere.
    let mut replacement: Vec<Option<String>> = vec![None; spans.len()];
    for (i, text) in texts.iter().enumerate() {
        if let Some(pos) = originals.iter().position(|o| o == text) {
            originals.remove(pos);
            replacement[i] = Some(text.clone());
        }
    }
    for (i, text) in texts.iter().enumerate() {
        if replacement[i].is_none()
            && let Some(pos) = originals.iter().position(|o| key(o) == key(text))
        {
            replacement[i] = Some(originals.remove(pos));
        }
    }

    let mut out = String::with_capacity(output.len());
    let mut last = 0;
    for (&(start, end), replacement) in spans.iter().zip(replacement) {
        out.extend(&chars[last..start]);
        match replacement {
            Some(text) => out.push_str(&text),
            None => out.extend(&chars[start..end]),
        }
        last = end;
    }
    out.extend(&chars[last..]);
    out
}

/// Character ranges of the single-quoted constants and quoted identifiers in
/// `text`, skipping comments and dollar-quoted bodies.
fn quoted_spans(text: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = text.chars().collect();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if let Some((span, end)) = scan(&chars, i) {
            let dollar = chars[i] == '$';
            if matches!(span, Span::Literal | Span::Identifier) && !dollar {
                spans.push((i, end));
            }
            i = end;
        } else {
            i += 1;
        }
    }
    spans
}

/// Whether moving the lines of `body` around -- re-indenting them, or joining
/// a one-line body onto new lines -- leaves every string constant and quoted
/// identifier in it unchanged.
///
/// It does unless one of them holds a newline, or one is unterminated and so
/// runs to the end. Scanning with a newline appended catches both: an
/// unterminated span swallows it.
pub(crate) fn layout_is_safe(body: &str) -> bool {
    let probe: Vec<char> = format!("{body}\n").chars().collect();
    let mut i = 0;
    while i < probe.len() {
        if let Some((span, end)) = scan(&probe, i) {
            if matches!(span, Span::Literal | Span::Identifier) && probe[i..end].contains(&'\n') {
                return false;
            }
            i = end;
        } else {
            i += 1;
        }
    }
    true
}

/// Put back the comments of `source` that `output` lost, or `None` when one of
/// them cannot be placed.
///
/// `comments` holds each comment's character offset in `source` and its text.
/// The formatters render the node kinds they know and never see comments, so
/// every one inside a statement was dropped. Each is re-attached at the end of
/// the output line that holds the token it followed in the source: the same
/// token, found by counting how many times it occurs before the comment.
/// Words compare case-insensitively, so keyword casing does not matter. When
/// the formatter rewrote that token (`int` to `INTEGER`, say) there is no
/// place to put the comment, and the caller falls back to the statement as
/// written. A comment the output already holds -- a passthrough path kept
/// it -- is left where it is. See https://github.com/gmr/libpgfmt/issues/62.
pub(crate) fn restore_comments(
    source: &str,
    comments: &[(usize, String)],
    output: &str,
) -> Option<String> {
    let source_tokens = match_tokens(&source.chars().collect::<Vec<_>>());
    let mut out = output.to_string();
    let mut present: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (_, text) in comments {
        present
            .entry(text.as_str())
            .or_insert_with(|| output.matches(text.as_str()).count());
    }
    let mut leading = Vec::new();
    for (offset, text) in comments {
        if let Some(n) = present.get_mut(text.as_str())
            && *n > 0
        {
            *n -= 1;
            continue;
        }
        let before: Vec<&str> = source_tokens
            .iter()
            .filter(|t| t.2 <= *offset)
            .map(|t| t.0.as_str())
            .collect();
        let Some(&anchor) = before.last() else {
            leading.push(text.clone());
            continue;
        };
        let nth = before.iter().filter(|t| **t == anchor).count();
        let chars: Vec<char> = out.chars().collect();
        let tokens = match_tokens(&chars);
        let &(_, _, end) = tokens.iter().filter(|t| t.0 == anchor).nth(nth - 1)?;
        let (at, line_has_comment) = line_end(&chars, end);
        let insert = if line_has_comment {
            // Never inside another line comment: give this one its own line.
            let start = chars[..at]
                .iter()
                .rposition(|c| *c == '\n')
                .map_or(0, |p| p + 1);
            let indent: String = chars[start..at].iter().take_while(|c| **c == ' ').collect();
            format!("\n{indent}{text}")
        } else {
            format!(" {text}")
        };
        out = chars[..at].iter().collect::<String>()
            + &insert
            + &chars[at..].iter().collect::<String>();
    }
    if !leading.is_empty() {
        out = format!("{}\n{out}", leading.join("\n"));
    }
    Some(out)
}

/// Words, constants and punctuation in `chars`, each with its character range,
/// skipping whitespace and comments. Words are lower-cased and constants have
/// their whitespace removed, so layout and keyword casing do not matter.
fn match_tokens(chars: &[char]) -> Vec<(String, usize, usize)> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if let Some((span, end)) = scan(chars, i) {
            if matches!(span, Span::Literal | Span::Identifier) {
                let text = chars[i..end]
                    .iter()
                    .filter(|c| !c.is_whitespace())
                    .collect();
                tokens.push((text, i, end));
            }
            i = end;
        } else if chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$' {
            let start = i;
            while i < chars.len()
                && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$')
            {
                i += 1;
            }
            tokens.push((
                chars[start..i].iter().collect::<String>().to_lowercase(),
                start,
                i,
            ));
        } else {
            if !chars[i].is_whitespace() {
                tokens.push((chars[i].to_string(), i, i + 1));
            }
            i += 1;
        }
    }
    tokens
}

/// The index of the first newline at or after `from` that is outside every
/// constant and comment, or the end of `chars`, and whether a line comment
/// sits between the two.
fn line_end(chars: &[char], from: usize) -> (usize, bool) {
    let mut has_comment = false;
    let mut i = from;
    while i < chars.len() {
        if let Some((span, end)) = scan(chars, i) {
            has_comment |= span == Span::LineComment;
            i = end;
        } else if chars[i] == '\n' {
            return (i, has_comment);
        } else {
            i += 1;
        }
    }
    (chars.len(), has_comment)
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
    fn restores_reindented_literals() {
        let source = "SELECT 'a\n b', 'x  y', 'same' FROM t";
        let output = "SELECT 'a\n     b',\n       'x  y',\n       'same'\n  FROM t";
        assert_eq!(
            restore_literals(source, output),
            "SELECT 'a\n b',\n       'x  y',\n       'same'\n  FROM t"
        );
        // A literal changed beyond whitespace is left alone.
        assert_eq!(restore_literals("SELECT 'a'", "SELECT 'b'"), "SELECT 'b'");
        // Dollar-quoted bodies are not touched.
        assert_eq!(restore_literals("$$a\nb$$", "$$a\n b$$"), "$$a\n b$$");
    }

    #[test]
    fn layout_safety() {
        assert!(layout_is_safe("\n  SELECT 'a' || $1;\n"));
        assert!(!layout_is_safe("x := 'line one\n  line two';"));
        assert!(!layout_is_safe("EXECUTE $q$\n SELECT 1 $q$;"));
        assert!(!layout_is_safe(" SELECT '(1,0')' + 1 "));
    }

    #[test]
    fn comments_are_reattached_to_their_line() {
        let source = "SELECT city FROM weather WHERE city LIKE 'S%' -- only S\nGROUP BY city";
        let offset = source.find("--").unwrap();
        let comments = [(source[..offset].chars().count(), "-- only S".to_string())];
        let output = "  SELECT city\n    FROM weather\n   WHERE city LIKE 'S%'\nGROUP BY city;";
        assert_eq!(
            restore_comments(source, &comments, output).unwrap(),
            "  SELECT city\n    FROM weather\n   WHERE city LIKE 'S%' -- only S\nGROUP BY city;"
        );
        // Already present: left alone.
        let kept = "SELECT 1 -- x";
        assert_eq!(
            restore_comments(kept, &[(9, "-- x".to_string())], kept).unwrap(),
            kept
        );
        // The anchor was rewritten: no place for the comment.
        assert!(
            restore_comments(
                "SELECT a::int -- x",
                &[(14, "-- x".to_string())],
                "SELECT a::INTEGER;"
            )
            .is_none()
        );
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
