pub mod lexer;
pub mod macro_expander;

pub struct Pos<T> {
    pub val: T,
    pub x: usize,
    pub y: usize,
}

impl<T> Pos<T> {
    pub fn new(val: T, x: usize, y: usize) -> Pos<T> {
        Pos { val, x, y }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Pos<U> {
        Pos::new(f(self.val), self.x, self.y)
    }
}

pub enum Mode {
    Text,
    Math,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
