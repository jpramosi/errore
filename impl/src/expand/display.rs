use proc_macro2::TokenStream;
use std::collections::BTreeSet as Set;

use quote::{format_ident, quote, ToTokens};
use syn::{DeriveInput, Ident, ImplGenerics, Member, Result, TypeGenerics};

use crate::ast::{DeriveType, Enum, Input, Struct};
use crate::attr::Trait;
use crate::generics::InferredBounds;
use crate::util::{fields_pat, use_as_display};

pub fn derive(input: &DeriveInput) -> TokenStream {
    match try_expand(input) {
        Ok(expanded) => expanded,
        // If there are invalid attributes in the input, expand to an Error impl
        // anyway to minimize spurious knock-on errors in other code that uses
        // this type as an Error.
        Err(error) => fallback(input, error),
    }
}

fn try_expand(input: &DeriveInput) -> Result<TokenStream> {
    let input = Input::from_syn(input, DeriveType::Display)?;
    input.validate()?;
    match input {
        Input::Enum(input) => Ok(impl_enum(input)),
        Input::Struct(input) => Ok(impl_struct(input)),
    }
}

fn fallback(input: &DeriveInput, error: syn::Error) -> TokenStream {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let error = error.to_compile_error();

    quote! {
        #error

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for #ty #ty_generics #where_clause {
            fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::unreachable!()
            }
        }
    }
}

fn impl_enum(input: Enum) -> TokenStream {
    let ty = &input.ident;
    let mut error = Option::<TokenStream>::None;
    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();

    for variant in &input.variants {
        if let Some(display) = &variant.attrs.display {
            error = display.recursing();
        }
    }

    let mut display_inferred_bounds = InferredBounds::new();
    let has_bonus_display = input.variants.iter().any(|v| {
        v.attrs
            .display
            .as_ref()
            .map_or(false, |display| display.has_bonus_display)
    });
    let use_as_display = use_as_display(has_bonus_display);
    let void_deref = if input.variants.is_empty() {
        Some(quote!(*))
    } else {
        None
    };

    let arms = input.variants.iter().map(|variant| {
        let ident = &variant.ident;
        let mut display_implied_bounds = Set::<(usize, Trait)>::new();
        let display = match &variant.attrs.display {
            Some(display) => {
                display_implied_bounds.clone_from(&display.implied_bounds);
                display.to_token_stream()
            }
            None => {
                if variant.fields.len() > 0 {
                    let only_field = match &variant.fields[0].member {
                        Member::Named(ident) => ident.clone(),
                        Member::Unnamed(index) => format_ident!("_{}", index),
                    };
                    display_implied_bounds.insert((0, Trait::Display));
                    quote!(::core::fmt::Display::fmt(#only_field, __formatter))
                } else {
                    // if no #[display("...")] is found, fallback to '<enum_name>::<enum_field_name>'
                    quote! {::core::fmt::Display::fmt(concat!(stringify!(#ty), "::", stringify!(#ident)), __formatter)}
                }
            }
        };
        for (field, bound) in display_implied_bounds {
            let field = &variant.fields[field];
            if field.contains_generic {
                display_inferred_bounds.insert(field.ty, bound);
            }
        }
        let pat = fields_pat(&variant.fields);
        quote! {
            #ty::#ident #pat => #display,
        }
    });

    let arms = arms.collect::<Vec<_>>();
    let display_where_clause = display_inferred_bounds.augment_where_clause(input.generics);
    let display_impl = quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for #ty #ty_generics #display_where_clause {
            fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                #use_as_display
                #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
                match #void_deref self {
                    #(#arms)*
                }
            }
        }
    };

    quote! {
        #error

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::error::Error for #ty #ty_generics #display_where_clause {}

        #display_impl
    }
}

pub fn impl_struct_display_body(
    input: &Struct,
    display_implied_bounds: &mut Set<(usize, Trait)>,
) -> Option<TokenStream> {
    return if input.attrs.transparent.is_some() {
        let only_field = &input.fields[0].member;
        display_implied_bounds.insert((0, Trait::Display));
        Some(quote! {
            ::core::fmt::Display::fmt(&self.#only_field, __formatter)
        })
    } else if let Some(display) = &input.attrs.display {
        display_implied_bounds.clone_from(&display.implied_bounds);
        let use_as_display = use_as_display(display.has_bonus_display);
        let pat = fields_pat(&input.fields);
        Some(quote! {
            #use_as_display
            #[allow(unused_variables, deprecated)]
            let Self #pat = self;
            #display
        })
    } else {
        None
    };
}

pub fn impl_struct_display(
    input: &Struct,
    ty: &Ident,
    ty_generics: &TypeGenerics<'_>,
    display_inferred_bounds: &mut InferredBounds,
    display_implied_bounds: Set<(usize, Trait)>,
    display_body: Option<TokenStream>,
    impl_generics: &ImplGenerics<'_>,
) -> Option<TokenStream> {
    for (field, bound) in display_implied_bounds {
        let field = &input.fields[field];
        if field.contains_generic {
            display_inferred_bounds.insert(field.ty, bound);
        }
    }
    let display_where_clause = display_inferred_bounds.augment_where_clause(input.generics);
    return display_body.as_ref().map(|body| {
        quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics ::core::fmt::Display for #ty #ty_generics #display_where_clause {
                #[allow(clippy::used_underscore_binding)]
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #body
                }
            }
        }
    });
}

fn impl_struct(input: Struct) -> TokenStream {
    let ty = &input.ident;
    let mut error = Option::<TokenStream>::None;
    let (impl_generics, ty_generics, _where_clause) = input.generics.split_for_impl();

    if let Some(display) = &input.attrs.display {
        error = display.recursing();
    }

    // implement display body
    let mut display_implied_bounds = Set::new();
    let mut display_body = impl_struct_display_body(&input, &mut display_implied_bounds);
    if display_body.is_none() {
        display_body = Some(quote! {
            ::core::fmt::Display::fmt(&format!("{:#?}", self), __formatter)
        });
    }

    // implement display
    let mut display_inferred_bounds = InferredBounds::new();
    let display_impl = impl_struct_display(
        &input,
        ty,
        &ty_generics,
        &mut display_inferred_bounds,
        display_implied_bounds,
        display_body,
        &impl_generics,
    );
    let display_where_clause = display_inferred_bounds.augment_where_clause(input.generics);

    let error_impl = quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics core::error::Error for #ty #ty_generics #display_where_clause {}
    };

    quote! {
        #error

        #display_impl
        #error_impl
    }
}
