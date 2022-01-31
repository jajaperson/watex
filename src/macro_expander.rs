use std::iter::FusedIterator;

use crate::{Pos, Token};

// TODO: Implement MacroExpander

/// Expands macros resulting in TeX that is completely token-free. TeX calls this the gullet.
pub struct ExpandMacros<I> {
    lexer: I,
}

impl<I: Iterator<Item = Pos<Token>>> ExpandMacros<I> {
    pub fn new(lexer: I) -> ExpandMacros<I> {
        ExpandMacros { lexer }
    }
}

impl<I: Iterator<Item = Pos<Token>>> Iterator for ExpandMacros<I> {
    type Item = Pos<Token>;

    fn next(&mut self) -> Option<Pos<Token>> {
        self.lexer.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.lexer.size_hint()
    }
}

impl<I> FusedIterator for ExpandMacros<I> where I: Iterator<Item = Pos<Token>> + FusedIterator {}

trait WithExpandMacros: Iterator<Item = Pos<Token>> + Sized {
    /// Expand each token in an iterator.
    fn expand_macros(self) -> ExpandMacros<Self> {
        ExpandMacros::new(self)
    }
}

impl<I> WithExpandMacros for I where I: Iterator<Item = Pos<Token>> {}
