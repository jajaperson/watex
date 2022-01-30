use std::iter::FusedIterator;

use crate::{Pos, Span};

pub struct PosChars<I>
where
    I: Iterator<Item = char>,
{
    chars: I,
    lin: usize,
    col: usize,
}

impl<I> Iterator for PosChars<I>
where
    I: Iterator<Item = char>,
{
    type Item = Pos<char>;

    fn next(&mut self) -> Option<Pos<char>> {
        self.chars.next().map(|ch| {
            let result = Pos::new(ch, Span::new(self.lin, self.col));
            if ch == '\n' {
                self.lin = 1;
                self.col += 1;
            } else {
                self.lin += 1;
            }
            result
        })
    }
}

impl<I> FusedIterator for PosChars<I> where I: Iterator<Item = char> + FusedIterator {}

pub trait WithPosChars: Iterator<Item = char> + Sized {
    fn with_pos(self) -> PosChars<Self> {
        PosChars {
            chars: self,
            lin: 1,
            col: 1,
        }
    }
}

impl<I: Iterator<Item = char> + Sized> WithPosChars for I {}
