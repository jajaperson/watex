use crate::lexer::Token;

// TODO: Implement MacroExpander

/// Expands macros resulting in TeX that is completely token-free. TeX calls this the gullet.
pub struct ExpandMacros<I> {
    lexer: I,
}

impl<I: Iterator<Item = Token>> ExpandMacros<I> {
    pub fn new(lexer: I) -> ExpandMacros<I> {
        ExpandMacros { lexer }
    }
}

impl<I: Iterator<Item = Token>> Iterator for ExpandMacros<I> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.lexer.next()
    }
}
