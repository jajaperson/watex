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
    // Illegal,
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
            // ...
            '\\' => Command(
                self.chars
                    .next()
                    .map_or("".into(), |nch| self.collect_command(nch, String::new())),
            ),
            '%' => Comment(self.build_comment(String::new())),
            '#' => Arg(self.build_arg(String::new())),
            _ if ch.is_whitespace() => Whitespace(self.collect_whitespace(ch, String::new())),
            _ => Char(ch),
        })
    }

    // Build states start from the first character

    /// Build a comment, starting from the current char
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

    /// Build an argument, starting from the current char.
    fn build_arg(&mut self, mut buffer: String) -> usize {
        if self.chars.peek().map_or(false, |ch| ch.is_ascii_digit()) {
            let ch = self.chars.next().expect("If None condition is false.");
            buffer.push(ch);
            self.build_arg(buffer)
        } else {
            buffer
                .parse()
                .expect("Only ascii digits should be collected in `buffer`")
        }
    }

    // Collect states must be provided with the first character

    /// Collects a command, starting from the provided char and continuing by iterating over `self.chars`.
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

    /// Collects whitespace, starting from the provided char (assumed to be whitespace) and continuing by iterating over `self.chars`
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
