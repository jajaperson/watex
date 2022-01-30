use std::{fmt, iter};

#[derive(Debug, PartialEq, Eq)]
pub enum Brace {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Command(String),
    Brace(Brace),
    Arg(usize),
    Ampersand,
    Whitespace(String),
    Comment(String),
    Char(char),
    Eof,
    Illegal,
}

pub struct Span {
    lin: usize,
    col: usize,
}

impl Span {
    pub fn new(lin: usize, col: usize) -> Span {
        Span { lin, col }
    }

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

pub struct Pos<T> {
    pub val: T,
    pub span: Span,
}

impl<T> Pos<T> {
    pub fn new(val: T, span: Span) -> Pos<T> {
        Pos { val, span }
    }

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
