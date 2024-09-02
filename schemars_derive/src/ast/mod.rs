mod from_serde;

use crate::attr::{ContainerAttrs, FieldAttrs, VariantAttrs};
use crate::idents::SCHEMA;
use from_serde::FromSerde;
use proc_macro2::TokenStream;
use serde_derive_internals::ast as serde_ast;
use serde_derive_internals::{Ctxt, Derive};

pub struct Container<'a> {
    pub ident: syn::Ident,
    pub serde_attrs: serde_derive_internals::attr::Container,
    pub data: Data<'a>,
    pub generics: syn::Generics,
    pub attrs: ContainerAttrs,
}

pub enum Data<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(serde_ast::Style, Vec<Field<'a>>),
}

pub struct Variant<'a> {
    pub ident: syn::Ident,
    pub serde_attrs: serde_derive_internals::attr::Variant,
    pub style: serde_ast::Style,
    pub fields: Vec<Field<'a>>,
    pub original: &'a syn::Variant,
    pub attrs: VariantAttrs,
}

pub struct Field<'a> {
    pub member: syn::Member,
    pub serde_attrs: serde_derive_internals::attr::Field,
    pub ty: &'a syn::Type,
    pub original: &'a syn::Field,
    pub attrs: FieldAttrs,
}

impl<'a> Container<'a> {
    pub fn from_ast(item: &'a syn::DeriveInput) -> syn::Result<Container<'a>> {
        let ctxt = Ctxt::new();
        let result = serde_ast::Container::from_ast(&ctxt, item, Derive::Deserialize)
            .ok_or(())
            .and_then(|serde| Self::from_serde(&ctxt, serde));

        ctxt.check()
            .map(|_| result.expect("from_ast set no errors on Ctxt, so should have returned Ok"))
    }

    pub fn name(&self) -> &str {
        self.serde_attrs.name().deserialize_name()
    }

    pub fn transparent_field(&'a self) -> Option<&'a Field> {
        if self.serde_attrs.transparent() {
            if let Data::Struct(_, fields) = &self.data {
                return Some(&fields[0]);
            }
        }

        None
    }

    pub fn add_mutators(&self, mutators: &mut Vec<TokenStream>) {
        self.attrs.common.add_mutators(mutators);
    }
}

impl<'a> Variant<'a> {
    pub fn name(&self) -> &str {
        self.serde_attrs.name().deserialize_name()
    }

    pub fn is_unit(&self) -> bool {
        matches!(self.style, serde_ast::Style::Unit)
    }

    pub fn add_mutators(&self, mutators: &mut Vec<TokenStream>) {
        self.attrs.common.add_mutators(mutators);
    }
}

impl<'a> Field<'a> {
    pub fn name(&self) -> &str {
        self.serde_attrs.name().deserialize_name()
    }

    pub fn add_mutators(&self, mutators: &mut Vec<TokenStream>) {
        self.attrs.common.add_mutators(mutators);
        self.attrs.validation.add_mutators(mutators);

        if self.serde_attrs.skip_deserializing() {
            mutators.push(quote! {
                schemars::_private::insert_metadata_property(&mut #SCHEMA, "readOnly", true);
            });
        }
        if self.serde_attrs.skip_serializing() {
            mutators.push(quote! {
                schemars::_private::insert_metadata_property(&mut #SCHEMA, "writeOnly", true);
            });
        }
    }
}
