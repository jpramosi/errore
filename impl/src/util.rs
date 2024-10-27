use proc_macro2::TokenStream;

use quote::{format_ident, quote};
use syn::{GenericArgument, Member, PathArguments, Type};

use crate::ast::Field;

pub fn use_as_display(needs_as_display: bool) -> Option<TokenStream> {
    if needs_as_display {
        Some(quote! {
            use errore::__private::AsDisplay as _;
        })
    } else {
        None
    }
}

pub fn from_initializer(from_field: &Field) -> TokenStream {
    let from_member = &from_field.member;
    let some_source = if type_is_option(from_field.ty) {
        quote!(::core::option::Option::Some(source))
    } else {
        quote!(source)
    };
    quote!({
        #from_member: #some_source,
    })
}

pub fn type_is_option(ty: &Type) -> bool {
    type_parameter_of_option(ty).is_some()
}

pub fn unoptional_type(ty: &Type) -> TokenStream {
    let unoptional = type_parameter_of_option(ty).unwrap_or(ty);
    quote!(#unoptional)
}

pub fn type_parameter_of_option(ty: &Type) -> Option<&Type> {
    let path = match ty {
        Type::Path(ty) => &ty.path,
        _ => return None,
    };

    let last = path.segments.last().unwrap();
    if last.ident != "Option" {
        return None;
    }

    let bracketed = match &last.arguments {
        PathArguments::AngleBracketed(bracketed) => bracketed,
        _ => return None,
    };

    if bracketed.args.len() != 1 {
        return None;
    }

    match &bracketed.args[0] {
        GenericArgument::Type(arg) => Some(arg),
        _ => None,
    }
}

pub fn fields_pat(fields: &[Field]) -> TokenStream {
    let mut members = fields.iter().map(|field| &field.member).peekable();
    match members.peek() {
        Some(Member::Named(_)) => quote!({ #(#members),* }),
        Some(Member::Unnamed(_)) => {
            let vars = members.map(|member| match member {
                Member::Unnamed(member) => format_ident!("_{}", member),
                Member::Named(_) => unreachable!(),
            });
            quote!((#(#vars),*))
        }
        None => quote!({}),
    }
}
