use std::{iter::Peekable, str::Chars};

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

/// Lexer (tokeniser) for latex maths mode code. TeX calls this the mouth.
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new<'b>(code: &'b str) -> Lexer<'b> {
        Lexer {
            chars: code.chars().peekable(),
        }
    }

    /// Get the next token
    fn next_token(&mut self) -> Option<Token> {
        use self::Brace::*;
        use self::Token::*;
        self.chars.next().map(|ch| match ch {
            '{' => Brace(Left),
            '}' => Brace(Right),
            '&' => Ampersand,
            '\0' => Eof,
            '\\' => Command(
                self.chars
                    .next()
                    .map_or("".into(), |nch| self.collect_command(nch, String::new())),
            ),
            '%' => Comment(self.build_comment(String::new())),
            '#' => self.chars.next().map_or(Illegal, |nch| {
                if nch.is_ascii_digit() {
                    Arg(self.collect_arg(nch, String::new()))
                } else {
                    Illegal
                }
            }),
            _ if ch.is_whitespace() => Whitespace(self.collect_whitespace(ch, String::new())),
            _ => Char(ch),
        })
    }

    // Build states start from the first character

    /// Builds a comment string, starting from the current char.
    fn build_comment(&mut self, mut buffer: String) -> String {
        match self.chars.next_if(|ch| *ch != '\n' && *ch != '\0') {
            Some(ch) => {
                buffer.push(ch);
                self.build_comment(buffer)
            }
            None => buffer,
        }
    }

    // Collect states must be provided with the first character

    /// Collects a command string, starting from the provided char and continuing by iterating over `self.chars`.
    fn collect_command(&mut self, current: char, mut buffer: String) -> String {
        buffer.push(current);
        match self.chars.next_if(|ch| ch.is_ascii_alphabetic()) {
            Some(ch) => self.collect_command(ch, buffer),
            None => buffer,
        }
    }

    /// Builds an argument integer, starting from the provided char (assumed to be an ascii digit) and continuing by iterating over `self.chars`.
    ///
    /// # Panics
    ///
    /// Panics if provided `current` is not a valid ascii digit.
    fn collect_arg(&mut self, current: char, mut buffer: String) -> usize {
        buffer.push(current);
        match self.chars.next_if(|ch| ch.is_ascii_digit()) {
            Some(ch) => self.collect_arg(ch, buffer),
            None => buffer
                .parse()
                .expect("`buffer` should only contain ASCII digits."),
        }
    }

    /// Collects whitespace string, starting from the provided char (assumed to be whitespace) and continuing by iterating over `self.chars`
    fn collect_whitespace(&mut self, current: char, mut buffer: String) -> String {
        buffer.push(current);
        match self.chars.next_if(|ch| ch.is_whitespace()) {
            Some(ch) => self.collect_whitespace(ch, buffer),
            None => buffer,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.next_token()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (chars_min, chars_max) = self.chars.size_hint();
        if chars_min == 0 {
            (0, chars_max)
        } else {
            (1, chars_max)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Brace::*, Lexer, Token::*};

    const EXAMPLE_LATEX: &str = r#"
\newcommand{\u}[1]{2^#1}
3^x &\geq\u{93\%} % I'm a comment.
#"#; // Final # is illegal

    #[test]
    fn parse_example() {
        let example_latex_tokenized = [
            Whitespace("\n".into()),
            Command("newcommand".into()),
            Brace(Left),
            Command("u".into()),
            Brace(Right),
            Char('['),
            Char('1'),
            Char(']'),
            Brace(Left),
            Char('2'),
            Char('^'),
            Arg(1),
            Brace(Right),
            Whitespace("\n".into()),
            Char('3'),
            Char('^'),
            Char('x'),
            Whitespace(" ".into()),
            Ampersand,
            Command("geq".into()),
            Command("u".into()),
            Brace(Left),
            Char('9'),
            Char('3'),
            Command("%".into()),
            Brace(Right),
            Whitespace(" ".into()),
            Comment(" I'm a comment.".into()),
            Whitespace("\n".into()),
            Illegal,
        ];

        let lexer = Lexer::new(EXAMPLE_LATEX);

        for (a, b) in lexer.zip(example_latex_tokenized) {
            assert_eq!(a, b);
        }
    }
}
