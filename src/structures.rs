use std::{fmt, iter};

/// Either left or right. Used to distinguish braces.
#[derive(Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// A single TeX token.
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    /// A TeX control sequence/macro, e.g. `\mathbb`, `\newcommand`, or `\%`.
    Control(String),
    /// A TeX brace, i.e. either `{` or `}`.
    Brace(Side),
    /// A TeX argument in a command definition, written as `#1`.
    Arg(usize),
    /// TeX's special alignment character `&`.
    Ampersand,
    /// A collection of consecutive Unicode whitespace characters.
    Whitespace(String),
    /// A TeX comment marked with `%`.
    Comment(String),
    /// A single, non-special character
    Char(char),
    /// The EOF character (`\0`).
    Eof,
    /// Represents an error encountered in the lexical token stream.
    Err(Error),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Emitted when an illegal character is encountered.
    IllegalChar(char),
}

/// Represents a position within a source file.
pub struct Span {
    lin: usize,
    col: usize,
}

impl Span {
    pub fn new(lin: usize, col: usize) -> Span {
        Span { lin, col }
    }

    /// Highlights the encoded position within a given body of text with `^`, followed by a message.
    pub fn highlight_msg_in_code(&self, code: &str, msg: &str) -> String {
        let mut result = String::new();
        let mut i = 1;
        for line in code.lines() {
            result += line;
            result += "\n";
            if i == self.lin {
                let padding: String = iter::repeat(' ').take(self.col - 1).collect();
                result = format!("{}{}^ {}\n", result, padding, msg);
            }
            i += 1;
        }
        result
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.lin, self.col)
    }
}

/// Some value enriched with a span.
pub struct Pos<T> {
    pub val: T,
    pub span: Span,
}

impl<T> Pos<T> {
    pub fn new(val: T, span: Span) -> Pos<T> {
        Pos { val, span }
    }

    /// Map over the contained value (i.e. without the span). Whatever is returned by `f` will
    /// inherit the original span.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Pos<U> {
        Pos::new(f(self.val), self.span)
    }
}

pub enum Mode {
    Text,
    Math,
}

#[cfg(test)]
mod tests {
    #[test]
    fn span_highlight_msg_in_code() {
        use crate::Span;

        let span = Span::new(2, 30);
        let test_code = r#"\begin{equation*}
    \textbf{illegal} \rarrow # \larrow \textbf{illegal}
\end{equation*}"#;

        let highlighted = span.highlight_msg_in_code(test_code, "Illegal character");
        let expected_highlighted: String = r#"\begin{equation*}
    \textbf{illegal} \rarrow # \larrow \textbf{illegal}
                             ^ Illegal character
\end{equation*}
"#
        .into();
        assert_eq!(highlighted, expected_highlighted);
    }
}
