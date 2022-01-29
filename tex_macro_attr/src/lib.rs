extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    braced, parenthesized,
    parse::{self, Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Mut},
    Attribute, Block, FnArg, Lit, ReturnType, Stmt, Token, Type, Visibility,
};

/// Marks a TeX macro function.
///
/// TODO: More documentation here.
#[proc_macro_attribute]
pub fn tex_macro(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    fn tex_macro2(attr: TokenStream, input: TokenStream) -> parse::Result<TokenStream> {
        let fun = parse2::<AbstractFun>(input)?;
        fun.validate()?;

        let name = if attr.is_empty() {
            fun.name.to_string_non_raw()
        } else {
            parse2::<Lit>(attr).and_then(|lit| match lit {
                Lit::Str(ls) => Ok(ls.value()),
                _ => Err(parse::Error::new(
                    lit.span(),
                    format_args!("Only takes string arguments, instead got {:?}", lit),
                )),
            })?
        };

        let macro_name = Ident::new(&name, fun.name.span());
        let fun_name = format_ident!("macro_{}", name);

        let AbstractFun {
            visibility,
            name: _,
            args,
            attributes,
            cooked,
            ret,
            body,
        } = fun;

        let tex_macro_path = quote!(watex::macros::TexMacro);

        Ok(quote! {
            #(#cooked)*
            #[allow(missing_docs)]
            #[allow(non_upper_case_globals)]
            pub static #macro_name: #tex_macro_path = #tex_macro_path {
                fun: #fun_name,
                names: &[#name], // TODO: Aliases
            };

            #(#cooked)*
            #(#attributes)*
            #[allow(missing_docs)]
            #[allow(non_snake_case)]
            #visibility fn #fun_name (#(#args),*) -> #ret {
                #(#body)*
            }
        })
    }

    tex_macro2(attr.into(), input.into())
        .unwrap_or_else(|err| TokenStream::from(err.to_compile_error()))
        .into()
}

/// An abstract structure to represent a parsed function/
struct AbstractFun {
    visibility: Visibility,
    name: Ident,
    args: Vec<Argument>,
    attributes: Vec<Attribute>,
    /// Special attributes that should be applied to associated structs.
    cooked: Vec<Attribute>,
    ret: Type,
    body: Vec<Stmt>,
}

impl AbstractFun {
    fn validate(&self) -> parse::Result<()> {
        // TODO: Validation step.
        const TEX_MACRO_MAX_ARGS: usize = 2;

        // Declaration
        if self.args.len() > TEX_MACRO_MAX_ARGS {
            return Err(parse::Error::new(
                self.args
                    .last()
                    .expect("Multiple arguments required")
                    .span(),
                format_args!(
                    "Function's arity exceeds more than {} arguments.",
                    TEX_MACRO_MAX_ARGS
                ),
            ));
        }

        // ...

        // Return type can't be validated since macros don't have access to the
        // type system

        Ok(())
    }
}

impl Parse for AbstractFun {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let (cooked, attributes) = partition_cooked(input.call(Attribute::parse_outer)?);

        // Walk through
        let visibility = input.parse::<Visibility>()?;
        input.parse::<Token![fn]>()?;
        let name = input.parse::<Ident>()?;
        println!("{}", name);
        let args = input
            .parse::<Parenthesized<FnArg>>()
            .map(|p| {
                let Parenthesized(args) = p;
                args
            })
            .and_then(|ars| {
                ars.into_iter()
                    .map(|ar| ar.try_into())
                    .collect::<parse::Result<Vec<Argument>>>()
            })?;
        let ret = input.parse::<ReturnType>().and_then(|r| match r {
            ReturnType::Type(_, t) => Ok((*t).clone()),
            ReturnType::Default => Err(input.error("Expected a result type of `TexResult`.")), // TODO: Work out what TeX macros are going to return
        })?;

        // Function body
        let content;
        braced!(content in input);
        let body = content.call(Block::parse_within)?;

        Ok(AbstractFun {
            visibility,
            name,
            args,
            attributes,
            cooked,
            ret,
            body,
        })
    }
}

struct Argument {
    mutability: Option<Mut>,
    name: Ident,
    typing: Type,
}

impl ToTokens for Argument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Argument {
            mutability,
            name,
            typing,
        } = self;

        tokens.extend(quote! {
            #mutability #name: #typing
        });
    }
}

impl TryInto<Argument> for FnArg {
    type Error = parse::Error;

    fn try_into(self) -> parse::Result<Argument> {
        match self {
            FnArg::Typed(typed) => {
                let pat = typed.pat;
                let typing = typed.ty;

                use syn::Pat;
                match *pat {
                    Pat::Ident(id) => {
                        let name = id.ident;
                        let mutability = id.mutability;

                        Ok(Argument {
                            mutability,
                            name,
                            typing: *typing,
                        })
                    }
                    Pat::Wild(wild) => {
                        let token = wild.underscore_token;
                        let name = Ident::new("_", token.spans[0]);

                        Ok(Argument {
                            mutability: None,
                            name,
                            typing: *typing,
                        })
                    }
                    _ => Err(parse::Error::new(
                        pat.span(),
                        format_args!("Unsupported pattern: {:?}", pat),
                    )),
                }
            }
            FnArg::Receiver(_) => Err(parse::Error::new(
                self.span(),
                format_args!("`self` arguments are prohibited: {:?}", self),
            )),
        }
    }
}

fn is_cooked(attr: &Attribute) -> bool {
    const COOKED_ATTRIBUTE_NAMES: &[&str] = &[
        "cfg", "cfg_attr", "derive", "inline", "allow", "warn", "deny", "forbid",
    ];

    COOKED_ATTRIBUTE_NAMES
        .iter()
        .any(|id| attr.path.is_ident(id))
}

fn partition_cooked(attrs: Vec<Attribute>) -> (Vec<Attribute>, Vec<Attribute>) {
    attrs.into_iter().partition(|attr| is_cooked(attr))
}

// Utils

#[derive(Debug)]
struct Parenthesized<T>(Punctuated<T, Comma>);

impl<T: Parse> Parse for Parenthesized<T> {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let content;
        parenthesized!(content in input);

        content
            .parse_terminated(T::parse)
            .map(|inner| Parenthesized(inner))
    }
}

trait ToStringNonRaw: Sized + ToString {
    fn to_string_non_raw(&self) -> String {
        self.to_string().trim_start_matches("r#").into()
    }
}

impl<T: Sized + ToString> ToStringNonRaw for T {}
