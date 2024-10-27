use proc_macro2::{Delimiter, Group, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::collections::BTreeSet as Set;

use quote::{format_ident, quote, ToTokens};
use syn::parse::discouraged::Speculative;
use syn::parse::ParseStream;
use syn::{
    braced, bracketed, parenthesized, token, Attribute, Error, Ident, Index, Lifetime, LitFloat,
    LitInt, LitStr, Meta, Result, Token,
};

#[derive(Default)]
pub struct Attrs<'a> {
    pub display: Option<Display<'a>>,
    pub source: Option<&'a Attribute>,
    pub from: Option<&'a Attribute>,
    pub transparent: Option<Transparent<'a>>,
    pub doc: Option<&'a Attribute>,
}

#[derive(Clone)]
pub struct Display<'a> {
    pub original: &'a Attribute,
    pub fmt: LitStr,
    pub args: TokenStream,
    pub requires_fmt_machinery: bool,
    pub has_bonus_display: bool,
    pub implied_bounds: Set<(usize, Trait)>,
    pub arg_tokens: Vec<Argument>,
}

impl<'a> Display<'a> {
    pub fn recursing(&self) -> Option<TokenStream> {
        for arg in &self.arg_tokens {
            match arg {
                // expressions with 'self' included (for e.g. in 'test_nested_display') doesn't work
                // need to check whether a function/expression is used
                // crate::attr::Argument::Ident(ident) => {
                //     if ident == "self" {
                //         return Some(
                //             syn::Error::new_spanned(ident, "cannot return without recursing")
                //                 .to_compile_error(),
                //         );
                //     }
                // }
                _ => {}
            };
        }
        None
    }
}

#[derive(Copy, Clone)]
pub struct Transparent<'a> {
    pub original: &'a Attribute,
    pub span: Span,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Trait {
    Debug,
    Display,
    Octal,
    LowerHex,
    UpperHex,
    Pointer,
    Binary,
    LowerExp,
    UpperExp,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum Argument {
    Ident(Ident),
    Lifetime(Lifetime),
    Literal(Literal),
    Punct(Punct),
}

pub fn get(input: &[Attribute]) -> Result<Attrs> {
    let mut attrs = Attrs::default();

    for attr in input {
        if attr.path().is_ident("error") && attr.path().is_ident("display") {
            return Err(Error::new_spanned(
                attr,
                "cannot specifiy #[error] and #[display] attribute at the same time",
            ));
        } else if attr.path().is_ident("error") {
            parse_display_attribute(&mut attrs, attr, "error")?;
        } else if attr.path().is_ident("display") {
            parse_display_attribute(&mut attrs, attr, "display")?;
        } else if attr.path().is_ident("source") {
            attr.meta.require_path_only()?;
            if attrs.source.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[source] attribute"));
            }
            attrs.source = Some(attr);
        } else if attr.path().is_ident("from") {
            match attr.meta {
                Meta::Path(_) => {}
                Meta::List(_) | Meta::NameValue(_) => {
                    // Assume this is meant for derive_more crate or something.
                    continue;
                }
            }
            if attrs.from.is_some() {
                return Err(Error::new_spanned(attr, "duplicate #[from] attribute"));
            }
            attrs.from = Some(attr);
        } else if attr.path().is_ident("doc") {
            attrs.doc = Some(attr);
        }
    }

    Ok(attrs)
}

fn parse_display_attribute<'a>(
    attrs: &mut Attrs<'a>,
    attr: &'a Attribute,
    name: &'static str,
) -> Result<()> {
    syn::custom_keyword!(transparent);

    attr.parse_args_with(|input: ParseStream| {
        if let Some(kw) = input.parse::<Option<transparent>>()? {
            if attrs.transparent.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    format!("duplicate #[{}(transparent)] attribute", name),
                ));
            }
            attrs.transparent = Some(Transparent {
                original: attr,
                span: kw.span,
            });
            return Ok(());
        }

        let fmt: LitStr = input.parse()?;

        let ahead = input.fork();
        ahead.parse::<Option<Token![,]>>()?;
        let mut arg_tokens = Vec::<Argument>::with_capacity(10);
        let args = if ahead.is_empty() {
            input.advance_to(&ahead);
            TokenStream::new()
        } else {
            parse_token_expr(input, false, &mut arg_tokens)?
        };

        let requires_fmt_machinery = !args.is_empty();

        let display = Display {
            original: attr,
            fmt,
            args,
            requires_fmt_machinery,
            has_bonus_display: false,
            implied_bounds: Set::new(),
            arg_tokens,
        };
        if attrs.display.is_some() {
            return Err(Error::new_spanned(
                attr,
                format!("only one #[{}(...)] attribute is allowed", name),
            ));
        }
        attrs.display = Some(display);
        Ok(())
    })
}

fn parse_token_expr(
    input: ParseStream,
    mut begin_expr: bool,
    arg_tokens: &mut Vec<Argument>,
) -> Result<TokenStream> {
    let mut tokens = Vec::new();
    while !input.is_empty() {
        if let Some((ident, _)) = input.cursor().ident() {
            arg_tokens.push(Argument::Ident(ident));
        } else if let Some((lifetime, _)) = input.cursor().lifetime() {
            arg_tokens.push(Argument::Lifetime(lifetime));
        } else if let Some((literal, _)) = input.cursor().literal() {
            arg_tokens.push(Argument::Literal(literal));
        } else if let Some((punct, _)) = input.cursor().punct() {
            arg_tokens.push(Argument::Punct(punct));
        }

        if begin_expr && input.peek(Token![.]) {
            if input.peek2(Ident) {
                input.parse::<Token![.]>()?;
                begin_expr = false;
                continue;
            } else if input.peek2(LitInt) {
                input.parse::<Token![.]>()?;
                let int: Index = input.parse()?;
                tokens.push({
                    let ident = format_ident!("_{}", int.index, span = int.span);
                    TokenTree::Ident(ident)
                });
                begin_expr = false;
                continue;
            } else if input.peek2(LitFloat) {
                let ahead = input.fork();
                ahead.parse::<Token![.]>()?;
                let float: LitFloat = ahead.parse()?;
                let repr = float.to_string();
                let mut indices = repr.split('.').map(syn::parse_str::<Index>);
                if let (Some(Ok(first)), Some(Ok(second)), None) =
                    (indices.next(), indices.next(), indices.next())
                {
                    input.advance_to(&ahead);
                    tokens.push({
                        let ident = format_ident!("_{}", first, span = float.span());
                        TokenTree::Ident(ident)
                    });
                    tokens.push({
                        let mut punct = Punct::new('.', Spacing::Alone);
                        punct.set_span(float.span());
                        TokenTree::Punct(punct)
                    });
                    tokens.push({
                        let mut literal = Literal::u32_unsuffixed(second.index);
                        literal.set_span(float.span());
                        TokenTree::Literal(literal)
                    });
                    begin_expr = false;
                    continue;
                }
            }
        }

        begin_expr = input.peek(Token![break])
            || input.peek(Token![continue])
            || input.peek(Token![if])
            || input.peek(Token![in])
            || input.peek(Token![match])
            || input.peek(Token![mut])
            || input.peek(Token![return])
            || input.peek(Token![while])
            || input.peek(Token![+])
            || input.peek(Token![&])
            || input.peek(Token![!])
            || input.peek(Token![^])
            || input.peek(Token![,])
            || input.peek(Token![/])
            || input.peek(Token![=])
            || input.peek(Token![>])
            || input.peek(Token![<])
            || input.peek(Token![|])
            || input.peek(Token![%])
            || input.peek(Token![;])
            || input.peek(Token![*])
            || input.peek(Token![-]);

        let token: TokenTree = if input.peek(token::Paren) {
            let content;
            let delimiter = parenthesized!(content in input);
            let nested = parse_token_expr(&content, true, arg_tokens)?;
            let mut group = Group::new(Delimiter::Parenthesis, nested);
            group.set_span(delimiter.span.join());
            TokenTree::Group(group)
        } else if input.peek(token::Brace) {
            let content;
            let delimiter = braced!(content in input);
            let nested = parse_token_expr(&content, true, arg_tokens)?;
            let mut group = Group::new(Delimiter::Brace, nested);
            group.set_span(delimiter.span.join());
            TokenTree::Group(group)
        } else if input.peek(token::Bracket) {
            let content;
            let delimiter = bracketed!(content in input);
            let nested = parse_token_expr(&content, true, arg_tokens)?;
            let mut group = Group::new(Delimiter::Bracket, nested);
            group.set_span(delimiter.span.join());
            TokenTree::Group(group)
        } else {
            input.parse()?
        };
        tokens.push(token);
    }
    Ok(TokenStream::from_iter(tokens))
}

impl ToTokens for Display<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fmt = &self.fmt;
        let args = &self.args;

        // Currently `write!(f, "text")` produces less efficient code than
        // `f.write_str("text")`. We recognize the case when the format string
        // has no braces and no interpolated values, and generate simpler code.
        tokens.extend(if self.requires_fmt_machinery {
            quote! {
                ::core::write!(__formatter, #fmt #args)
            }
        } else {
            quote! {
                __formatter.write_str(#fmt)
            }
        });
    }
}

impl ToTokens for Trait {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let trait_name = match self {
            Trait::Debug => "Debug",
            Trait::Display => "Display",
            Trait::Octal => "Octal",
            Trait::LowerHex => "LowerHex",
            Trait::UpperHex => "UpperHex",
            Trait::Pointer => "Pointer",
            Trait::Binary => "Binary",
            Trait::LowerExp => "LowerExp",
            Trait::UpperExp => "UpperExp",
        };
        let ident = Ident::new(trait_name, Span::call_site());
        tokens.extend(quote!(::core::fmt::#ident));
    }
}
