use proc_macro2::TokenStream;
use std::collections::BTreeSet as Set;

use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse_quote, DeriveInput, Generics, Ident, ImplGenerics, Member, Result, Token, TypeGenerics,
    Visibility, WhereClause,
};

use crate::ast::{DeriveType, Enum, Field, Input, Struct, Variant};
use crate::attr::{Attrs, Trait};
use crate::expand::display;
use crate::generics::InferredBounds;
use crate::span::MemberSpan;
use crate::util::{fields_pat, from_initializer, type_is_option, unoptional_type, use_as_display};

pub fn derive(input: &DeriveInput) -> TokenStream {
    match try_expand(input) {
        Ok(expanded) => expanded,
        // If there are invalid attributes in the input, expand to an Error impl
        // anyway to minimize spurious knock-on errors in other code that uses
        // this type as an Error.
        // Basically this function tries to reduce other errors
        // that are caused by the actual error by creating
        // a dummy implementation.
        Err(error) => fallback(input, error),
    }
}

fn try_expand(input: &DeriveInput) -> Result<TokenStream> {
    let input = Input::from_syn(input, DeriveType::Error)?;
    input.validate()?;
    match input {
        Input::Enum(input) => impl_enum(input),
        Input::Struct(input) => impl_struct(input),
    }
}

fn fallback(input: &DeriveInput, error: syn::Error) -> TokenStream {
    let ty = &input.ident;
    let vis = input.vis.to_token_stream();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let error = error.to_compile_error();
    let static_lifetime = if input.generics.lifetimes().count() > 0 {
        Some(quote! {<'static>})
    } else {
        Some(ty_generics.to_token_stream())
    };

    quote! {
        #error

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::error::Error for #ty #ty_generics #where_clause
        where
            // Work around trivial bounds being unstable.
            // https://github.com/rust-lang/rust/issues/48214
            for<'workaround> #ty #ty_generics: ::core::fmt::Debug,
        {}

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for #ty #ty_generics #where_clause {
            fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::unreachable!()
            }
        }

        extern crate alloc;

        #[allow(unused_qualifications)]
        #[automatically_derived]
        #[derive(Debug)]
        #vis struct Ec #ty_generics (pub errore::span::Span<#ty #ty_generics>) #where_clause;

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics errore::Metadata for #ty #ty_generics #where_clause {
            fn name(&self) -> &'static str {
                ::core::unreachable!()
            }

            fn id(&self) -> &'static errore::Id {
                ::core::unreachable!()
            }

            fn target(&self) -> &'static str {
                ::core::unreachable!()
            }

            fn target_id(&self) -> &'static errore::Id {
                ::core::unreachable!()
            }

            fn display(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::unreachable!()
            }

            fn is_transparent(&self) -> bool {
                ::core::unreachable!()
            }
        }

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::convert::From<#ty #static_lifetime> for Ec #ty_generics #where_clause {
            fn from(value: #ty #static_lifetime) -> Self {
                ::core::unreachable!()
            }
        }
    }
}

#[derive(Debug)]
pub enum StaticVariable {
    CtorName,
    CtorId,
    CtorTarget,
    CtorTargetId,
    LazyName,
    LazyId,
    LazyTarget,
    LazyTargetId,
}

fn get_var(var_type: StaticVariable, ty: &Ident, ident: Option<&Ident>) -> TokenStream {
    syn::parse_str::<TokenStream>(&format!(
        "__{:?}{}{}",
        var_type,
        ty,
        ident.map(|f| f.to_string()).unwrap_or(String::new())
    ))
    .unwrap()
}

fn access_static_var(
    ctor: StaticVariable,
    lazy: StaticVariable,
    ty: &Ident,
    ident: Option<&Ident>,
) -> TokenStream {
    let ctor_var = get_var(ctor, ty, ident);
    let lazy_var = get_var(lazy, ty, ident);

    // access static variable defined with impl_static_var
    quote! {{
        errore::__private::access_static_var!(#ctor_var, #lazy_var)
    }}
}

fn impl_static_var<'a>(ty: &'a Ident, mut field: Option<&'a Ident>) -> TokenStream {
    // Miri doesn't handle link sections! -> https://github.com/rust-lang/miri/issues/450
    // Moreover some platforms are not supported by crates that utilize link sections.
    // For example 'riscv32imac-unknown-none-elf' fails to build with 'rust-ctor'.
    // That means code that interacts with the `rust-ctor` or `inventory` crate will either fail
    // with miri or doesn't build at all on some exotic platforms.
    // Unfortunately errore depends strongly on these variables and excluding the tests is therefore
    // not possible.
    // To workaround this issue, lazy static variables are compiled
    // if the 'miri' configuration flag is set or the 'ctor' feature is disabled.
    let ctor_name = get_var(StaticVariable::CtorName, ty, field);
    let ctor_id = get_var(StaticVariable::CtorId, ty, field);
    let ctor_target = get_var(StaticVariable::CtorTarget, ty, field);
    let ctor_target_id = get_var(StaticVariable::CtorTargetId, ty, field);
    let lazy_name = get_var(StaticVariable::LazyName, ty, field);
    let lazy_id = get_var(StaticVariable::LazyId, ty, field);
    let lazy_target = get_var(StaticVariable::LazyTarget, ty, field);
    let lazy_target_id = get_var(StaticVariable::LazyTargetId, ty, field);
    if field.is_none() {
        field = Some(ty);
    }

    let block_name = quote! {{
        let mut module_path = module_path!();
        let mut id = alloc::string::String::with_capacity(module_path.len() + 10);
        alloc::format!(
            "{}::{}::{}",
            env!("CARGO_PKG_NAME").replace("-", "_"),
            module_path.rsplit("::").next().unwrap(),
            stringify!(#field)
        )
    }};

    let block_id = quote! {{
        errore::Id::from(
            errore::__private::fnv1a_hash_64(
                concat!(
                    env!("CARGO_PKG_NAME"),
                    module_path!(),
                    stringify!(#ty),
                    stringify!(#field)
                ).as_bytes()
            )
        )
    }};

    let block_target = quote! {{
        let mut module_path = module_path!();
        alloc::format!(
            "{}",
            module_path.split("::").next().unwrap(),
        )
    }};

    let block_target_id = quote! {{
        let mut module_path = module_path!();
        errore::Id::from(
            errore::__private::fnv1a_hash_64(
                alloc::format!(
                    "{}",
                    module_path.split("::").next().unwrap(),
                ).as_bytes()
            )
        )
    }};

    quote! {
        errore::__private::impl_static_var!(#ctor_name, #lazy_name, alloc::string::String, #block_name);
        errore::__private::impl_static_var!(#ctor_id, #lazy_id, errore::Id, #block_id);
        errore::__private::impl_static_var!(#ctor_target, #lazy_target, alloc::string::String, #block_target);
        errore::__private::impl_static_var!(#ctor_target_id, #lazy_target_id, errore::Id, #block_target_id);
    }
}

fn is_error_context(token: &TokenStream) -> bool {
    // It is possible to also use the feature 'error_generic_member_access'
    // instead of using a keyword here.
    // However, the api is not matured enough and doesn't provide methods to access and
    // mutate the error itself.
    // Moreover the relevant fields need to be wrapped in a 'Arc<Mutex<T>>' type to be useful.
    let token_str = token.to_string();
    token_str.ends_with(":: Ec") || token_str.contains(":: Ec <")
}

fn impl_from<'a>(
    ty: &'a Ident,
    ty_generics: &'a TypeGenerics,
    where_clause: &'a Option<&WhereClause>,
    impl_generics: &'a ImplGenerics,
    from_field: &'a Field,
    variant: Option<&Ident>,
) -> TokenStream {
    let from = unoptional_type(from_field.ty);
    let body = from_initializer(from_field);
    let error = match variant {
        Some(v) => quote! { #ty::#v #body },
        None => quote! { #ty #body },
    };

    let from_str = from.to_string();
    let ty_str = ty.to_string();
    quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::convert::From<#from> for #ty #ty_generics #where_clause {
            #[track_caller]
            fn from(source: #from) -> Self {
                errore::dlog!(
                    "From<{}> for {}::{}",
                    #from_str,
                    module_path!(),
                    #ty_str
                );
                #error
            }
        }
    }
}

fn impl_enum(input: Enum) -> Result<TokenStream> {
    let ty = &input.ident;
    let mut error = Option::<TokenStream>::None;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut error_inferred_bounds = InferredBounds::new();

    let lazy_vars_arms = input
        .variants
        .iter()
        .map(|variant| impl_static_var(ty, Some(&variant.ident)));
    let lazy_vars = quote! {
        #(#lazy_vars_arms)*
    };

    for variant in &input.variants {
        if let Some(display) = &variant.attrs.display {
            error = display.recursing();
        }
    }

    let source_method = if input.has_source() {
        let arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            if let Some(transparent_attr) = &variant.attrs.transparent {
                let only_field = &variant.fields[0];
                if only_field.contains_generic {
                    error_inferred_bounds.insert(only_field.ty, quote!(::core::error::Error));
                }
                let member = &only_field.member;
                let source = quote_spanned! {transparent_attr.span=>
                    ::core::error::Error::source(transparent.as_dyn_error())
                };
                quote! {
                    #ty::#ident {#member: transparent} => #source,
                }
            } else if let Some(source_field) = variant.source_field() {
                let source = &source_field.member;
                if source_field.contains_generic {
                    let ty = unoptional_type(source_field.ty);
                    error_inferred_bounds.insert(ty, quote!(::core::error::Error + 'static));
                }
                let asref = if type_is_option(source_field.ty) {
                    Some(quote_spanned!(source.member_span()=> .as_ref()?))
                } else {
                    None
                };
                let varsource = quote!(source);
                let dyn_error = quote_spanned! {source_field.source_span()=>
                    #varsource #asref.as_dyn_error()
                };
                quote! {
                    #ty::#ident {#source: #varsource, ..} => ::core::option::Option::Some(#dyn_error),
                }
            } else {
                quote! {
                    #ty::#ident {..} => ::core::option::Option::None,
                }
            }
        });
        Some(quote! {
            fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
                use errore::__private::AsDynError as _;
                #[allow(deprecated)]
                match self {
                    #(#arms)*
                }
            }
        })
    } else {
        None
    };

    let display_impl = if input.has_display() {
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
            let mut display_implied_bounds = Set::new();
            let display = match &variant.attrs.display {
                Some(display) => {
                    display_implied_bounds.clone_from(&display.implied_bounds);
                    display.to_token_stream()
                }
                None => {
                    let only_field = match &variant.fields[0].member {
                        Member::Named(ident) => ident.clone(),
                        Member::Unnamed(index) => format_ident!("_{}", index),
                    };
                    display_implied_bounds.insert((0, Trait::Display));
                    quote!(::core::fmt::Display::fmt(#only_field, __formatter))
                }
            };
            for (field, bound) in display_implied_bounds {
                let field = &variant.fields[field];
                if field.contains_generic {
                    display_inferred_bounds.insert(field.ty, bound);
                }
            }
            let ident = &variant.ident;
            let pat = fields_pat(&variant.fields);
            quote! {
                #ty::#ident #pat => #display
            }
        });

        let arms = arms.collect::<Vec<_>>();
        let display_where_clause = display_inferred_bounds.augment_where_clause(input.generics);
        Some(quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics ::core::fmt::Display for #ty #ty_generics #display_where_clause {
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    #use_as_display
                    #[allow(unused_variables, deprecated, clippy::used_underscore_binding)]
                    match #void_deref self {
                        #(#arms,)*
                    }
                }
            }
        })
    } else {
        None
    };

    let from_impls = input.variants.iter().filter_map(|variant| {
        let from_field = variant.from_field()?;
        let variant = &variant.ident;
        Some(impl_from(
            ty,
            &ty_generics,
            &where_clause,
            &impl_generics,
            from_field,
            Some(variant),
        ))
    });

    let metadata_impl = {
        let name_arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let var = access_static_var(
                StaticVariable::CtorName,
                StaticVariable::LazyName,
                ty,
                Some(ident),
            );
            quote! {
                #ty::#ident {..} => #var,
            }
        });

        let id_arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let var = access_static_var(
                StaticVariable::CtorId,
                StaticVariable::LazyId,
                ty,
                Some(ident),
            );
            quote! {
                #ty::#ident {..} => #var,
            }
        });

        let target_arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let var = access_static_var(
                StaticVariable::CtorTarget,
                StaticVariable::LazyTarget,
                ty,
                Some(ident),
            );
            quote! {
                #ty::#ident {..} => #var,
            }
        });

        let target_id_arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let var = access_static_var(
                StaticVariable::CtorTargetId,
                StaticVariable::LazyTargetId,
                ty,
                Some(ident),
            );
            quote! {
                #ty::#ident {..} => #var,
            }
        });

        let transparent_arms = input.variants.iter().map(|variant| {
            let ident = &variant.ident;
            let is_transparent = variant.attrs.transparent.is_some();
            quote! {
                #ty::#ident {..} => #is_transparent,
            }
        });

        Some(quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics errore::Metadata for #ty #ty_generics #where_clause {
                fn name(&self) -> &'static str {
                    match self {
                        #(#name_arms)*
                    }
                }

                fn id(&self) -> &'static errore::Id {
                    match self {
                        #(#id_arms)*
                    }
                }

                fn target(&self) -> &'static str {
                    match self {
                        #(#target_arms)*
                    }
                }

                fn target_id(&self) -> &'static errore::Id {
                    match self {
                        #(#target_id_arms)*
                    }
                }

                #[inline]
                fn display(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::fmt::Display::fmt(self, f)
                }

                fn is_transparent(&self) -> bool {
                    match self {
                        #(#transparent_arms)*
                    }
                }
            }
        })
    };

    let impl_extractable = quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics errore::Extractable for #ty #ty_generics #where_clause {}
    };

    if input.generics.type_params().next().is_some() {
        let self_token = <Token![Self]>::default();
        error_inferred_bounds.insert(self_token, Trait::Debug);
        error_inferred_bounds.insert(self_token, Trait::Display);
    }
    let error_where_clause = error_inferred_bounds.augment_where_clause(input.generics);
    let impl_error = {
        quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics ::core::error::Error for #ty #ty_generics #error_where_clause {
                #source_method
            }
        }
    };

    let from_fields = input
        .variants
        .iter()
        .filter_map(|variant| Some(variant.from_field()?))
        .collect::<Vec<&Field>>();
    let impl_error_context = impl_error_context(
        &input.ident,
        &input.vis,
        input.generics,
        &input.attrs,
        Some(&input.variants),
        from_fields,
    );

    Ok(quote! {
        #error

        #lazy_vars
        #display_impl
        #metadata_impl
        #(#from_impls)*
        #impl_extractable
        #impl_error
        #impl_error_context
    })
}

fn impl_struct(input: Struct) -> Result<TokenStream> {
    let ty = &input.ident;
    let mut error = Option::<TokenStream>::None;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut error_inferred_bounds = InferredBounds::new();

    let lazy_vars = impl_static_var(ty, None);

    if let Some(display) = &input.attrs.display {
        error = display.recursing();
    }

    let source_body = if let Some(transparent_attr) = &input.attrs.transparent {
        let only_field = &input.fields[0];
        if only_field.contains_generic {
            error_inferred_bounds.insert(only_field.ty, quote!(::core::error::Error));
        }
        let member = &only_field.member;
        Some(quote_spanned! {transparent_attr.span=>
            ::core::error::Error::source(self.#member.as_dyn_error())
        })
    } else if let Some(source_field) = input.source_field() {
        let source = &source_field.member;
        if source_field.contains_generic {
            let ty = unoptional_type(source_field.ty);
            error_inferred_bounds.insert(ty, quote!(::core::error::Error + 'static));
        }
        let asref = if type_is_option(source_field.ty) {
            Some(quote_spanned!(source.member_span()=> .as_ref()?))
        } else {
            None
        };
        let dyn_error = quote_spanned! {source_field.source_span()=>
            self.#source #asref.as_dyn_error()
        };
        Some(quote! {
            ::core::option::Option::Some(#dyn_error)
        })
    } else {
        None
    };
    let source_method = source_body.map(|body| {
        quote! {
            fn source(&self) -> ::core::option::Option<&(dyn ::core::error::Error + 'static)> {
                use errore::__private::AsDynError as _;
                #body
            }
        }
    });

    // implement display body
    let mut display_implied_bounds = Set::new();
    let display_body = display::impl_struct_display_body(&input, &mut display_implied_bounds);

    // implement display
    let mut display_inferred_bounds = InferredBounds::new();
    let display_impl = display::impl_struct_display(
        &input,
        ty,
        &ty_generics,
        &mut display_inferred_bounds,
        display_implied_bounds,
        display_body,
        &impl_generics,
    );

    let metadata_impl = {
        let var_name =
            access_static_var(StaticVariable::CtorName, StaticVariable::LazyName, ty, None);
        let var_id = access_static_var(StaticVariable::CtorId, StaticVariable::LazyId, ty, None);
        let var_target = access_static_var(
            StaticVariable::CtorTarget,
            StaticVariable::LazyTarget,
            ty,
            None,
        );
        let var_target_id = access_static_var(
            StaticVariable::CtorTargetId,
            StaticVariable::LazyTargetId,
            ty,
            None,
        );
        let is_transparent = input.attrs.transparent.is_some();
        quote! {
            #[allow(unused_qualifications)]
            #[automatically_derived]
            impl #impl_generics errore::Metadata for #ty #ty_generics #where_clause {
                #[inline]
                fn name(&self) -> &'static str {
                    #var_name
                }

                #[inline]
                fn id(&self) -> &'static errore::Id {
                    #var_id
                }

                #[inline]
                fn target(&self) -> &'static str {
                    #var_target
                }

                #[inline]
                fn target_id(&self) -> &'static errore::Id {
                    #var_target_id
                }

                #[inline]
                fn display(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    ::core::fmt::Display::fmt(self, f)
                }

                #[inline]
                fn is_transparent(&self) -> bool {
                    #is_transparent
                }
            }
        }
    };

    let from_impl = input.from_field().map(|from_field| {
        Some(impl_from(
            ty,
            &ty_generics,
            &where_clause,
            &impl_generics,
            from_field,
            None,
        ))
    });

    let impl_extractable = quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics errore::Extractable for #ty #ty_generics #where_clause {}
    };

    if input.generics.type_params().next().is_some() {
        let self_token = <Token![Self]>::default();
        error_inferred_bounds.insert(self_token, Trait::Debug);
        error_inferred_bounds.insert(self_token, Trait::Display);
    }
    let error_where_clause = error_inferred_bounds.augment_where_clause(input.generics);
    let impl_error = quote! {
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::error::Error for #ty #ty_generics #error_where_clause {
            #source_method
        }
    };

    let mut from_fields = Vec::<&Field>::new();
    if let Some(from_field) = input.from_field() {
        from_fields.push(from_field);
    }
    let impl_error_context = impl_error_context(
        &input.ident,
        &input.vis,
        input.generics,
        &input.attrs,
        None,
        from_fields,
    );

    Ok(quote! {
        #error

        #lazy_vars
        #display_impl
        #metadata_impl
        #from_impl
        #impl_extractable
        #impl_error
        #impl_error_context
    })
}

fn impl_error_context(
    ty: &Ident,
    vis: &Visibility,
    generics: &Generics,
    attrs: &Attrs,
    variants: Option<&Vec<Variant>>,
    from_fields: Vec<&Field>,
) -> TokenStream {
    let vis = vis.to_token_stream();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut iter_generics = generics.clone();
    iter_generics.params.push(parse_quote!('iter));
    let (iter_impl_generics, _iter_ty_generics, iter_where_clause) = iter_generics.split_for_impl();
    let ty_str = ty.to_string();

    let static_lifetime = if generics.lifetimes().count() > 0 {
        Some(quote! {<'static>})
    } else {
        Some(ty_generics.to_token_stream())
    };

    let doc = if attrs.doc.is_some() {
        let mut from_doc = String::with_capacity(128);
        for from_field in &from_fields {
            let from = unoptional_type(from_field.ty).to_string().replace(" ", "");
            from_doc.push_str(&format!("- [{}]\n\n", from));
        }
        if !from_doc.is_empty() {
            from_doc = format!(
                "\n\nThe [`From`](std::convert::From) trait is implemented for:\n{}",
                from_doc
            )
        }

        let doc_str = if variants.is_some() {
            format!("Context for [`{}`](enum.{}.html).{}", ty, ty, from_doc)
        } else {
            format!("Context for [`{}`](struct.{}.html).{}", ty, ty, from_doc)
        };

        Some(quote! {
            #[doc = #doc_str]
        })
    } else {
        None
    };

    let from_impls = from_fields.iter().map(|from_field| {
        let from = unoptional_type(from_field.ty);
        let from_str = from.to_string();

        // conversion from error context
        if is_error_context(&from) {
            Some(quote! {
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::convert::From<#from> for Ec {
                    #[track_caller]
                    fn from(mut value: #from) -> Self {
                        errore::dlog!(
                            "From<{}> for {}::Ec",
                            #from_str,
                            module_path!()
                        );
                        let ctx = value.take_trace();
                        Self(errore::span::Span::new(ctx, #ty::from(value)))
                    }
                }
            })
        } else {
            Some(quote! {
                #[allow(unused_qualifications)]
                #[automatically_derived]
                impl ::core::convert::From<#from> for Ec {
                    #[track_caller]
                    fn from(value: #from) -> Self {
                        errore::dlog!(
                            "From<{}> for {}::Ec",
                            #from_str,
                            module_path!()
                        );
                        let ctx = None;
                        Self(errore::span::Span::new(ctx, #ty::from(value)))
                    }
                }
            })
        }
    });

    quote! {
        extern crate alloc;

        #doc
        #[allow(unused_qualifications)]
        #[automatically_derived]
        #[derive(Debug)]
        #vis struct Ec #ty_generics (#[doc(hidden)] pub errore::span::Span<#ty #ty_generics>) #where_clause;

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics Ec #ty_generics #where_clause {
            #[track_caller]
            pub fn new(kind: #ty #static_lifetime) -> Self {
                Self(errore::span::Span::new(None, kind))
            }

            /// Returns the inherited error with its actual type.
            #[inline]
            pub fn error(&self) -> &#ty #ty_generics {
                self.0.inner.as_ref()
            }
        }

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for Ec #ty_generics #where_clause {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                self.0.fmt(f)
            }
        }

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::error::Error for Ec #static_lifetime #where_clause {
            #[inline]
            fn source(&self) -> Option<&(dyn ::core::error::Error + 'static)> {
                Some(&self.0 as &dyn ::core::error::Error)
            }
        }

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #iter_impl_generics IntoIterator for &'iter Ec #static_lifetime #iter_where_clause {
            type Item = &'iter errore::TraceRecord;

            type IntoIter = errore::__private::TraceRecordIterator<'iter>;

            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        // Forwards metadata implementation to the inner enum or struct error.
        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics errore::Metadata for Ec #static_lifetime #where_clause {
            #[inline]
            fn name(&self) -> &'static str {
                self.0.inner.name()
            }

            #[inline]
            fn id(&self) -> &'static errore::Id {
                self.0.inner.id()
            }

            #[inline]
            fn target(&self) -> &'static str {
                self.0.inner.target()
            }

            #[inline]
            fn target_id(&self) -> &'static errore::Id {
                self.0.inner.target_id()
            }

            #[inline]
            fn display(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(self, f)
            }

            #[inline]
            fn is_transparent(&self) -> bool {
                self.0.inner.is_transparent()
            }
        }

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics errore::Extract for Ec #ty_generics #where_clause {
            #[inline]
            fn get<'a, E>(&'a self) -> Option<errore::Downcasted<'a, E>>
            where
                E: ::core::error::Error + errore::Extractable + 'static,
            {
                self.0.get::<E>()
            }

            #[inline]
            fn has<'a, E>(&'a self) -> bool
            where
                E: ::core::error::Error + errore::Extractable + 'static,
            {
                self.0.has::<E>()
            }
        }

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics errore::Traceable for Ec #ty_generics #where_clause {
            #[inline]
            fn trace(&self) -> &errore::TraceContext {
                self.0.ctx.as_ref().expect("Trace should be available in 'Traceable::trace'")
            }

            #[inline]
            fn trace_ref(&self) -> Option<&errore::TraceContext> {
                self.0.ctx.as_ref()
            }

            #[inline]
            fn take_trace(&mut self) -> Option<errore::TraceContext> {
                self.0.ctx.take()
            }

            #[inline]
            fn inner(&self) -> alloc::sync::Arc<dyn ::core::error::Error + ::core::marker::Send + ::core::marker::Sync> {
                return self.0.inner.clone();
            }

            fn insert(&mut self, mut record: errore::TraceRecord) -> bool {
                let ctx = self.0.ctx.as_mut().expect("Trace should be available in 'Traceable::insert'");

                errore::__private::for_each_subscriber(|s| s.on_try_record(&mut errore::span::SpanContext::new(
                    ctx,
                    &record,
                )));

                // keep in sync with src/span.rs
                if ctx.insert(record) {
                    errore::__private::for_each_subscriber(|s| s.on_record(ctx));
                    return true;
                }

                return false;
            }
        }

        #[allow(unused_qualifications)]
        #[automatically_derived]
        // Used to convert an enum field or struct to context.
        impl #impl_generics ::core::convert::From<#ty #static_lifetime> for Ec #ty_generics #where_clause {
            #[track_caller]
            fn from(value: #ty #static_lifetime) -> Self {
                errore::dlog!(
                    "From<{}::{}> for {}::Ec",
                    module_path!(),
                    #ty_str,
                    module_path!()
                );
                Self(errore::span::Span::new(None, value))
            }
        }

        #(#from_impls)*
    }
}
