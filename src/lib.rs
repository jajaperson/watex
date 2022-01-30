pub mod lexer;
pub mod macro_expander;
pub mod macros;

mod structures;
mod util;

pub use structures::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
