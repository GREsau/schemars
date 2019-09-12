use quote::ToTokens;
use serde_derive_internals::Ctxt;
use std::collections::BTreeSet;
use syn::parse::Parser;
use syn::{
    Attribute, Data, DeriveInput, Field, GenericParam, Generics, Ident, Meta, NestedMeta, Variant,
};

pub fn add_trait_bounds(generics: &mut Generics) {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(schemars::JsonSchema));
        }
    }
}

// If a struct/variant/field has any #[schemars] attributes, then rename them
// to #[serde] so that serde_derive_internals will parse them for us.
pub fn process_serde_attrs(input: &mut DeriveInput) -> Result<(), String> {
    let ctxt = Ctxt::new();
    process_attrs(&ctxt, &mut input.attrs);
    match input.data {
        Data::Struct(ref mut s) => process_serde_field_attrs(&ctxt, s.fields.iter_mut()),
        Data::Enum(ref mut e) => process_serde_variant_attrs(&ctxt, e.variants.iter_mut()),
        Data::Union(ref mut u) => process_serde_field_attrs(&ctxt, u.fields.named.iter_mut()),
    };

    ctxt.check()
}

fn process_serde_variant_attrs<'a>(ctxt: &Ctxt, variants: impl Iterator<Item = &'a mut Variant>) {
    for v in variants {
        process_attrs(&ctxt, &mut v.attrs);
        process_serde_field_attrs(&ctxt, v.fields.iter_mut());
    }
}

fn process_serde_field_attrs<'a>(ctxt: &Ctxt, fields: impl Iterator<Item = &'a mut Field>) {
    for f in fields {
        process_attrs(&ctxt, &mut f.attrs);
    }
}

fn process_attrs(ctxt: &Ctxt, attrs: &mut Vec<Attribute>) {
    let mut serde_meta: Vec<NestedMeta> = attrs
        .iter()
        .filter(|a| a.path.is_ident("serde"))
        .flat_map(|attr| get_meta_items(&ctxt, attr))
        .flatten()
        .collect();

    attrs.retain(|a| a.path.is_ident("schemars"));

    for attr in attrs.iter_mut() {
        let schemars_ident = attr.path.segments.pop().unwrap().into_value().ident;
        attr.path
            .segments
            .push(Ident::new("serde", schemars_ident.span()).into());
    }

    let mut schemars_meta_names: BTreeSet<String> = attrs
        .iter()
        .flat_map(|attr| get_meta_items(&ctxt, attr))
        .flatten()
        .flat_map(|m| get_meta_ident(&ctxt, &m))
        .map(|i| i.to_string())
        .collect();
    if schemars_meta_names.contains("with") {
        schemars_meta_names.insert("serialize_with".to_string());
        schemars_meta_names.insert("deserialize_with".to_string());
    }

    serde_meta.retain(|m| {
        get_meta_ident(&ctxt, m)
            .map(|i| !schemars_meta_names.contains(&i.to_string()))
            .unwrap_or(false)
    });

    if serde_meta.is_empty() {
        return;
    }

    let new_serde_attr = quote! {
        #[serde(#(#serde_meta),*)]
    };

    let parser = Attribute::parse_outer;
    match parser.parse2(new_serde_attr) {
        Ok(ref mut parsed) => attrs.append(parsed),
        Err(e) => ctxt.error(e),
    }
}

fn get_meta_items(ctxt: &Ctxt, attr: &Attribute) -> Result<Vec<NestedMeta>, ()> {
    match attr.parse_meta() {
        Ok(Meta::List(meta)) => Ok(meta.nested.into_iter().collect()),
        Ok(_) => {
            ctxt.error("expected #[schemars(...)] or #[serde(...)]");
            Err(())
        }
        Err(err) => {
            ctxt.error(err);
            Err(())
        }
    }
}

fn get_meta_ident(ctxt: &Ctxt, meta: &NestedMeta) -> Result<Ident, ()> {
    match meta {
        NestedMeta::Meta(m) => Ok(m.name()),
        NestedMeta::Literal(lit) => {
            ctxt.error(format!(
                "unexpected literal in attribute: {}",
                lit.into_token_stream()
            ));
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use syn::DeriveInput;

    #[test]
    fn test_process_serde_attrs() {
        let mut input: DeriveInput = parse_quote! {
            #[serde(container, container2 = "blah")]
            #[serde(container3(foo, bar))]
            #[schemars(container2 = "overridden", container4)]
            #[misc]
            struct MyStruct {
                #[serde(field, field2)]
                field1: i32,
                #[serde(field, field2, serialize_with = "se", deserialize_with = "de")]
                #[schemars(field = "overridden", with = "with")]
                field2: i32,
                #[schemars(field)]
                field3: i32,
            }
        };
        let expected: DeriveInput = parse_quote! {
            #[serde(container2 = "overridden", container4)]
            #[serde(container, container3(foo, bar))]
            struct MyStruct {
                #[serde(field, field2)]
                field1: i32,
                #[serde(field = "overridden", with = "with")]
                #[serde(field2)]
                field2: i32,
                #[serde(field)]
                field3: i32,
            }
        };

        if let Err(e) = process_serde_attrs(&mut input) {
            panic!("process_serde_attrs returned error: {}", e)
        };

        assert_eq!(input, expected);
    }
}
