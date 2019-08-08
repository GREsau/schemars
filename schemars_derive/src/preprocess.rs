use proc_macro2::Span;
use syn::{Attribute, Data, DeriveInput, Field, GenericParam, Generics, Ident, Variant};

pub fn add_trait_bounds(generics: &mut Generics) {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(schemars::MakeSchema));
        }
    }
}

// If a struct/variant/field has any #[schemars] attributes, then rename them
// to #[serde] so that serde_derive_internals will parse them for us.
pub fn rename_schemars_attrs(input: &mut DeriveInput) {
    rename_attrs(input.attrs.iter_mut());
    match input.data {
        Data::Struct(ref mut s) => rename_field_attrs(s.fields.iter_mut()),
        Data::Enum(ref mut e) => rename_variant_attrs(e.variants.iter_mut()),
        Data::Union(ref mut u) => rename_field_attrs(u.fields.named.iter_mut()),
    };
}

fn rename_variant_attrs<'a>(variants: impl Iterator<Item = &'a mut Variant>) {
    for v in variants {
        rename_attrs(v.attrs.iter_mut());
        rename_field_attrs(v.fields.iter_mut());
    }
}

fn rename_field_attrs<'a>(fields: impl Iterator<Item = &'a mut Field>) {
    for f in fields {
        rename_attrs(f.attrs.iter_mut());
    }
}

fn rename_attrs<'a>(attrs: impl Iterator<Item = &'a mut Attribute>) {
    let (schemars_attrs, others): (Vec<_>, Vec<_>) =
        attrs.partition(|a| a.path.is_ident("schemars"));

    if !schemars_attrs.is_empty() {
        for attr in schemars_attrs {
            let schemars_ident = attr.path.segments.pop().unwrap().into_value().ident;
            attr.path
                .segments
                .push(Ident::new("serde", schemars_ident.span()).into());
        }
        // Give any other attributes a new name so that serde doesn't process them
        // and complain about duplicate attributes.
        // TODO we shouldn't need to remove all attributes - it should be possible
        // to just remove duplicated parts of the attributes.
        for attr in others {
            attr.path
                .segments
                .push(Ident::new("dummy", Span::call_site()).into());
        }
    }
}
