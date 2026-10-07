//! The XCL lexer: turn one source line into tokens.
//!
//! Rules (see the language spec):
//! * a `#` at line-start (after trimming) is a full-line comment;
//! * tokens are whitespace-separated, but `"..."` (with backslash escapes) and
//!   `'...'` (raw, no escapes) group spaces into one token;
//! * a token may embed `$name` / `{BUILTIN}` references, which the parser turns
//!   into [`crate::xcl::ast::Arg`] values.

/// The result of lexing one physical line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    /// Empty / whitespace-only — skipped.
    Blank,
    /// A full-line comment.
    Comment,
    /// A statement: its tokens (already unquoted).
    Statement(Vec<String>),
}

/// A lex error with a 1-based column for reporting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub column: usize,
    pub message: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "column {}: {}", self.column, self.message)
    }
}

/// Lexes a single line into [`Line`].
pub fn lex_line(source: &str) -> Result<Line, LexError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Ok(Line::Blank);
    }
    if trimmed.starts_with('#') {
        return Ok(Line::Comment);
    }

    let bytes: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0usize;

    while index < bytes.len() {
        // Skip whitespace.
        if bytes[index].is_whitespace() {
            index += 1;
            continue;
        }

        let start = index;
        let mut token = String::new();
        match bytes[index] {
            '"' => {
                index += 1;
                loop {
                    if index >= bytes.len() {
                        return Err(LexError {
                            column: start,
                            message: "unterminated double-quoted string".into(),
                        });
                    }
                    match bytes[index] {
                        '"' => {
                            index += 1;
                            break;
                        }
                        '\\' => {
                            index += 1;
                            if index >= bytes.len() {
                                return Err(LexError {
                                    column: index,
                                    message: "dangling backslash".into(),
                                });
                            }
                            token.push(match bytes[index] {
                                '"' => '"',
                                '\\' => '\\',
                                'n' => '\n',
                                't' => '\t',
                                other => other,
                            });
                            index += 1;
                        }
                        c => {
                            token.push(c);
                            index += 1;
                        }
                    }
                }
            }
            '\'' => {
                index += 1;
                loop {
                    if index >= bytes.len() {
                        return Err(LexError {
                            column: start,
                            message: "unterminated single-quoted string".into(),
                        });
                    }
                    if bytes[index] == '\'' {
                        index += 1;
                        break;
                    }
                    token.push(bytes[index]);
                    index += 1;
                }
            }
            _ => {
                // Bare token: until whitespace.
                while index < bytes.len() && !bytes[index].is_whitespace() {
                    token.push(bytes[index]);
                    index += 1;
                }
            }
        }
        tokens.push(token);
    }

    Ok(Line::Statement(tokens))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_bare_tokens() {
        assert_eq!(
            lex_line("open https://example.com").unwrap(),
            Line::Statement(vec!["open".into(), "https://example.com".into()])
        );
    }

    #[test]
    fn lexes_quoted_tokens_with_spaces() {
        assert_eq!(
            lex_line("fill \"#name\" \"Ada Lovelace\"").unwrap(),
            Line::Statement(vec!["fill".into(), "#name".into(), "Ada Lovelace".into()])
        );
    }

    #[test]
    fn lexes_single_quotes_without_escapes() {
        assert_eq!(
            lex_line("request POST url '{\"a\":1}'").unwrap(),
            Line::Statement(vec![
                "request".into(),
                "POST".into(),
                "url".into(),
                "{\"a\":1}".into()
            ])
        );
    }

    #[test]
    fn treats_full_line_comments() {
        assert_eq!(lex_line("   # a comment").unwrap(), Line::Comment);
    }

    #[test]
    fn collapses_whitespace_runs() {
        assert_eq!(
            lex_line("  click  3  ").unwrap(),
            Line::Statement(vec!["click".into(), "3".into()])
        );
    }

    #[test]
    fn rejects_unterminated_quotes() {
        assert!(lex_line("fill \"oops").is_err());
    }
}
