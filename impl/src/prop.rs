use proc_macro2::Span;
use syn::Member;

use crate::ast::{Enum, Field, Struct, Variant};
use crate::span::MemberSpan;

impl Enum<'_> {
    pub(crate) fn has_source(&self) -> bool {
        self.variants
            .iter()
            .any(|variant| variant.source_field().is_some() || variant.attrs.transparent.is_some())
    }

    pub(crate) fn has_display(&self) -> bool {
        self.attrs.display.is_some()
            || self.attrs.transparent.is_some()
            || self
                .variants
                .iter()
                .any(|variant| variant.attrs.display.is_some())
            || self
                .variants
                .iter()
                .all(|variant| variant.attrs.transparent.is_some())
    }
}

impl Struct<'_> {
    pub(crate) fn from_field(&self) -> Option<&Field> {
        from_field(&self.fields)
    }

    pub(crate) fn source_field(&self) -> Option<&Field> {
        source_field(&self.fields)
    }
}

impl Variant<'_> {
    pub(crate) fn from_field(&self) -> Option<&Field> {
        from_field(&self.fields)
    }

    pub(crate) fn source_field(&self) -> Option<&Field> {
        source_field(&self.fields)
    }
}

impl Field<'_> {
    pub(crate) fn source_span(&self) -> Span {
        if let Some(source_attr) = &self.attrs.source {
            source_attr.path().get_ident().unwrap().span()
        } else if let Some(from_attr) = &self.attrs.from {
            from_attr.path().get_ident().unwrap().span()
        } else {
            self.member.member_span()
        }
    }
}

fn from_field<'a, 'b>(fields: &'a [Field<'b>]) -> Option<&'a Field<'b>> {
    for field in fields {
        if field.attrs.from.is_some() {
            return Some(field);
        }
    }
    None
}

fn source_field<'a, 'b>(fields: &'a [Field<'b>]) -> Option<&'a Field<'b>> {
    for field in fields {
        if field.attrs.from.is_some() || field.attrs.source.is_some() {
            return Some(field);
        }
    }
    for field in fields {
        match &field.member {
            Member::Named(ident) if ident == "source" => return Some(field),
            _ => {}
        }
    }
    None
}
