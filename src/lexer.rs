use std::{iter::Peekable, str::Chars};

use crate::Pos;

pub struct PosChars<I>
where
    I: Iterator<Item = char>,
{
    chars: I,
    x: usize,
    y: usize,
}

impl<I> Iterator for PosChars<I>
where
    I: Iterator<Item = char>,
{
    type Item = Pos<char>;

    fn next(&mut self) -> Option<Pos<char>> {
        self.chars.next().map(|ch| {
            let result = Pos::new(ch, self.x, self.y);
            if ch == '\n' {
                self.x = 1;
                self.y += 1;
            } else {
                self.x += 1;
            }
            result
        })
    }
}

pub trait CreatePosChars: Iterator<Item = char> + Sized {
    fn with_pos(self) -> PosChars<Self>;
}

impl<I: Iterator<Item = char> + Sized> CreatePosChars for I {
    fn with_pos(self) -> PosChars<Self> {
        PosChars {
            chars: self,
            x: 1,
            y: 1,
        }
    }
}

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
pub struct Lexer<I>
where
    I: Iterator<Item = char>,
{
    chars: Peekable<PosChars<I>>,
}

impl<'a> Lexer<Chars<'a>> {
    pub fn new(code: &'a str) -> Lexer<Chars<'a>> {
        Lexer {
            chars: code.chars().with_pos().peekable(),
        }
    }
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    /// Get the next token
    fn next_token(&mut self) -> Option<Pos<Token>> {
        use self::Brace::*;
        use self::Token::*;
        self.chars.next().map(|ch| Pos {
            val: match ch.val {
                '{' => Brace(Left),
                '}' => Brace(Right),
                '&' => Ampersand,
                '\0' => Eof,
                '\\' => Command(
                    self.chars
                        .next()
                        .map_or("".into(), |ch| self.collect_command(ch.val, String::new())),
                ),
                '%' => Comment(self.build_comment(String::new())),
                '#' => self.chars.next().map_or(Illegal, |ch| {
                    if ch.val.is_ascii_digit() {
                        Arg(self.collect_arg(ch.val, String::new()))
                    } else {
                        Illegal
                    }
                }),
                _ if ch.val.is_whitespace() => {
                    Whitespace(self.collect_whitespace(ch.val, String::new()))
                }
                _ => Char(ch.val),
            },
            x: ch.x,
            y: ch.y,
        })
    }

    // Build states start from the first character

    /// Builds a comment string, starting from the current char.
    fn build_comment(&mut self, mut buffer: String) -> String {
        match self.chars.next_if(|ch| ch.val != '\n' && ch.val != '\0') {
            Some(ch) => {
                buffer.push(ch.val);
                self.build_comment(buffer)
            }
            None => buffer,
        }
    }

    // Collect states must be provided with the first character

    /// Collects a command string, starting from the provided char and continuing by iterating over `self.chars`.
    fn collect_command(&mut self, current: char, mut buffer: String) -> String {
        buffer.push(current);
        match self.chars.next_if(|ch| ch.val.is_ascii_alphabetic()) {
            Some(ch) => self.collect_command(ch.val, buffer),
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
        match self.chars.next_if(|ch| ch.val.is_ascii_digit()) {
            Some(ch) => self.collect_arg(ch.val, buffer),
            None => buffer
                .parse()
                .expect("`buffer` should only contain ASCII digits."),
        }
    }

    /// Collects whitespace string, starting from the provided char (assumed to be whitespace) and continuing by iterating over `self.chars`
    fn collect_whitespace(&mut self, current: char, mut buffer: String) -> String {
        buffer.push(current);
        match self.chars.next_if(|ch| ch.val.is_whitespace()) {
            Some(ch) => self.collect_whitespace(ch.val, buffer),
            None => buffer,
        }
    }
}

impl<I> Iterator for Lexer<I>
where
    I: Iterator<Item = char>,
{
    type Item = Pos<Token>;

    fn next(&mut self) -> Option<Pos<Token>> {
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

        for (a, b) in lexer.map(|ch| ch.val).zip(example_latex_tokenized) {
            assert_eq!(a, b);
        }
    }
}
