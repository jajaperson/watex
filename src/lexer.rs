use std::{
    iter::{FusedIterator, Peekable},
    str::Chars,
};

use crate::{
    util::{PosChars, WithPosChars},
    Error, Pos, Side, Token,
};

/// Lexer (tokeniser) for latex maths mode code. TeX calls this the mouth. The public interface is
/// an iterator over the lexed tokens.
pub struct Lexer<I>
where
    I: Iterator<Item = char>,
{
    chars: Peekable<PosChars<I>>,
}

impl<'a> Lexer<Chars<'a>> {
    /// Create a lexer for a given `&str`.
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
        use Side::*;
        use Token::*;
        self.chars.next().map(|pch| {
            pch.map(|ch| match ch {
                '{' => Brace(Left),
                '}' => Brace(Right),
                '&' => Ampersand,
                '\0' => Eof,
                '\\' => Control(self.chars.next().map_or("".into(), |pch| {
                    self.collect_command(pch.val, String::new())
                })),
                '%' => Comment(self.build_comment(String::new())),
                '#' => self
                    .chars
                    .next()
                    .map_or(Err(Error::IllegalChar('#')), |pch| {
                        if pch.val.is_ascii_digit() {
                            Arg(self.collect_arg(pch.val, String::new()))
                        } else {
                            Err(Error::IllegalChar('#'))
                        }
                    }),
                _ if ch.is_whitespace() => Whitespace(self.collect_whitespace(ch, String::new())),
                _ => Char(ch),
            })
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

impl<I> FusedIterator for Lexer<I> where I: Iterator<Item = char> + FusedIterator {}

#[cfg(test)]
mod tests {
    use crate::{Error, Lexer, Side::*, Token::*};

    const EXAMPLE_LATEX: &str = r#"
\newcommand{\u}[1]{2^#1}
3^x &\geq\u{93\%} % I'm a comment.
#"#; // Final # is illegal

    #[test]
    fn parse_example() {
        let example_latex_tokenized = [
            Whitespace("\n".into()),
            Control("newcommand".into()),
            Brace(Left),
            Control("u".into()),
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
            Control("geq".into()),
            Control("u".into()),
            Brace(Left),
            Char('9'),
            Char('3'),
            Control("%".into()),
            Brace(Right),
            Whitespace(" ".into()),
            Comment(" I'm a comment.".into()),
            Whitespace("\n".into()),
            Err(Error::IllegalChar('#')),
        ];

        let lexer = Lexer::new(EXAMPLE_LATEX);

        for (a, b) in lexer.map(|ch| ch.val).zip(example_latex_tokenized) {
            assert_eq!(a, b);
        }
    }
}
