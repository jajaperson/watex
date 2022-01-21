use std::{iter::Peekable, str::Chars};

pub enum Brace {
    Left,
    Right,
}

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

/// Lexer (tokeniser) for latex maths mode code.
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
        if self
            .chars
            .peek()
            .map_or(true, |ch| *ch == '\n' || *ch == '\0')
        {
            buffer
        } else {
            let ch = self.chars.next().expect("If None condition is true.");
            buffer.push(ch);
            self.build_comment(buffer)
        }
    }

    // Collect states must be provided with the first character

    /// Collects a command string, starting from the provided char and continuing by iterating over `self.chars`.
    fn collect_command(&mut self, current: char, mut buffer: String) -> String {
        buffer.push(current);
        if self
            .chars
            .peek()
            .map_or(false, |ch| ch.is_ascii_alphabetic())
        {
            let nch = self.chars.next().expect("If None condition is false.");
            self.collect_command(nch, buffer)
        } else {
            buffer
        }
    }

    /// Builds an argument integer, starting from the provided char (assumed to be an ascii digit) and continuing by iterating over `self.chars`.
    ///
    /// # Panics
    ///
    /// Panics if provided `current` is not a valid ascii digit.
    fn collect_arg(&mut self, current: char, mut buffer: String) -> usize {
        buffer.push(current);
        if self.chars.peek().map_or(false, |ch| ch.is_ascii_digit()) {
            let nch = self.chars.next().expect("If None condition is false.");
            self.collect_arg(nch, buffer)
        } else {
            buffer
                .parse()
                .expect("`buffer` should only contain ascii digits.")
        }
    }

    /// Collects whitespace string, starting from the provided char (assumed to be whitespace) and continuing by iterating over `self.chars`
    fn collect_whitespace(&mut self, current: char, mut buffer: String) -> String {
        buffer.push(current);
        if self.chars.peek().map_or(false, |ch| ch.is_whitespace()) {
            let nch = self.chars.next().expect("If None condition is false.");
            self.collect_whitespace(nch, buffer)
        } else {
            buffer
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
