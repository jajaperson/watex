pub struct TexMacroCtx {}
pub struct TexMacroResult {}

/// Underlying function for a procedural TeX macro.
pub type TexMacroFn = fn(TexMacroCtx) -> TexMacroResult;

/// Represents a procedural TeX macro.
pub struct TexMacro {
    pub fun: TexMacroFn,
    pub names: &'static [&'static str],
}
