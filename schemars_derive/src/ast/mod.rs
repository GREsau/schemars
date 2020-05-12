mod from_serde;

use crate::attr::{Attrs, WithAttr};
use from_serde::FromSerde;
use serde_derive_internals::ast as serde_ast;
use serde_derive_internals::{Ctxt, Derive};

pub struct Container<'a> {
    pub ident: syn::Ident,
    pub serde_attrs: serde_derive_internals::attr::Container,
    pub data: Data<'a>,
    pub generics: &'a syn::Generics,
    pub original: &'a syn::DeriveInput,
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
    pub attrs: Attrs,
}

pub struct Field<'a> {
    pub member: syn::Member,
    pub serde_attrs: serde_derive_internals::attr::Field,
    pub ty: &'a syn::Type,
    pub original: &'a syn::Field,
    pub attrs: Attrs,
}

impl<'a> Container<'a> {
    pub fn from_ast(item: &'a syn::DeriveInput) -> Result<Container<'a>, Vec<syn::Error>> {
        let ctxt = Ctxt::new();
        let result = serde_ast::Container::from_ast(&ctxt, item, Derive::Deserialize)
            .ok_or(())
            .and_then(|serde| Self::from_serde(&ctxt, serde));

        ctxt.check()
            .map(|_| result.expect("from_ast set no errors on Ctxt, so should have returned Ok"))
    }

    pub fn name(&self) -> String {
        self.serde_attrs.name().deserialize_name()
    }
}

impl<'a> Variant<'a> {
    pub fn name(&self) -> String {
        self.serde_attrs.name().deserialize_name()
    }

    pub fn is_unit(&self) -> bool {
        match self.style {
            serde_ast::Style::Unit => true,
            _ => false,
        }
    }
}

impl<'a> Field<'a> {
    pub fn name(&self) -> String {
        self.serde_attrs.name().deserialize_name()
    }

    pub fn type_for_schema(&self) -> &syn::Type {
        match &self.attrs.with {
            None => self.ty,
            Some(WithAttr::Type(ty)) => ty,
            Some(WithAttr::_Function(_)) => unimplemented!(), // TODO
        }
    }
}
