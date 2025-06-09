use crate::{
    ast::{Container, Data, Field, Variant},
    attr::WithAttr,
};
use std::collections::BTreeSet;
use syn::{punctuated::Punctuated, Ident};

// This logic is heavily based on serde_derive:
// https://github.com/serde-rs/serde/blob/a1ddb18c92f32d64b2ccaf31ddd776e56be34ba2/serde_derive/src/bound.rs#L91

/// Returns a tuple of:
/// 1. The `where` clause to be included on the `JsonSchema` impl
/// 2. A `BTreeSet` of type params that are "relevant" to the impl, i.e. excluding params only used
///    in `PhantomData` or skipped fields
pub fn find_trait_bounds<'a>(
    cont: &'a Container<'a>,
) -> (Option<syn::WhereClause>, BTreeSet<&'a Ident>) {
    if cont.generics.params.is_empty() {
        return (cont.generics.where_clause.clone(), BTreeSet::new());
    }

    let all_type_params =
        BTreeSet::from_iter(cont.generics.type_params().map(|param| &param.ident));

    assert!(cont.rename_type_params.is_subset(&all_type_params));

    let mut visitor = FindTyParams {
        all_type_params,
        relevant_type_params: cont.rename_type_params.clone(),
    };

    if visitor.all_type_params.len() > visitor.relevant_type_params.len() {
        match &cont.data {
            Data::Enum(variants) => {
                for variant in variants {
                    let relevant_fields = variant
                        .fields
                        .iter()
                        .filter(|field| needs_jsonschema_bound(field, Some(variant)));
                    for field in relevant_fields {
                        visitor.visit_field(field);
                    }
                }
            }
            Data::Struct(_, fields) => {
                let relevant_fields = fields
                    .iter()
                    .filter(|field| needs_jsonschema_bound(field, None));
                for field in relevant_fields {
                    visitor.visit_field(field);
                }
            }
        }
    }

    let mut where_clause = cont
        .generics
        .where_clause
        .clone()
        .unwrap_or_else(|| syn::WhereClause {
            where_token: <Token![where]>::default(),
            predicates: Punctuated::default(),
        });

    if let Some(bounds) = cont.serde_attrs.de_bound() {
        where_clause.predicates.extend(bounds.iter().cloned());
    } else {
        where_clause
            .predicates
            .extend(visitor.relevant_type_params.iter().map(|ty| {
                syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: Punctuated::from_iter([syn::PathSegment {
                                ident: (*ty).clone(),
                                arguments: syn::PathArguments::None,
                            }]),
                        },
                    }),
                    colon_token: <Token![:]>::default(),
                    bounds: Punctuated::from_iter([syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: parse_quote!(schemars::JsonSchema),
                    })]),
                })
            }));
    }

    (Some(where_clause), visitor.relevant_type_params)
}

fn needs_jsonschema_bound(field: &Field, variant: Option<&Variant>) -> bool {
    if let Some(variant) = variant {
        if variant.serde_attrs.skip_deserializing() && variant.serde_attrs.skip_serializing() {
            return false;
        }
    }
    if field.serde_attrs.skip_deserializing() && field.serde_attrs.skip_serializing() {
        return false;
    }

    true
}

struct FindTyParams<'ast> {
    all_type_params: BTreeSet<&'ast Ident>,
    relevant_type_params: BTreeSet<&'ast Ident>,
}

impl<'ast> FindTyParams<'ast> {
    fn visit_field(&mut self, field: &'ast Field) {
        match &field.attrs.with {
            Some(WithAttr::Type(ty)) => self.visit_type(ty),
            Some(WithAttr::Function(f)) => self.visit_path(f),
            None => self.visit_type(&field.original.ty),
        }
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(seg) = path.segments.last() {
            if seg.ident == "PhantomData" {
                // Hardcoded exception, because PhantomData<T> implements
                // JsonSchema whether or not T implements it.
                return;
            }
        }

        if path.leading_colon.is_none() {
            if let Some(first_segment) = path.segments.first() {
                let id = &first_segment.ident;
                if self.all_type_params.contains(id) {
                    self.relevant_type_params.insert(id);
                }
            }
        }

        for segment in &path.segments {
            self.visit_path_segment(segment);
        }
    }

    fn visit_type(&mut self, ty: &'ast syn::Type) {
        match ty {
            syn::Type::Array(ty) => self.visit_type(&ty.elem),
            syn::Type::BareFn(ty) => {
                for arg in &ty.inputs {
                    self.visit_type(&arg.ty);
                }
                self.visit_return_type(&ty.output);
            }
            syn::Type::Group(ty) => self.visit_type(&ty.elem),
            syn::Type::ImplTrait(ty) => {
                for bound in &ty.bounds {
                    self.visit_type_param_bound(bound);
                }
            }
            syn::Type::Macro(ty) => self.visit_macro(&ty.mac),
            syn::Type::Paren(ty) => self.visit_type(&ty.elem),
            syn::Type::Path(ty) => {
                if let Some(qself) = &ty.qself {
                    self.visit_type(&qself.ty);
                }
                self.visit_path(&ty.path);
            }
            syn::Type::Ptr(ty) => self.visit_type(&ty.elem),
            syn::Type::Reference(ty) => {
                self.visit_type(&ty.elem);
            }
            syn::Type::Slice(ty) => self.visit_type(&ty.elem),
            syn::Type::TraitObject(ty) => {
                for bound in &ty.bounds {
                    self.visit_type_param_bound(bound);
                }
            }
            syn::Type::Tuple(ty) => {
                for elem in &ty.elems {
                    self.visit_type(elem);
                }
            }

            syn::Type::Infer(_) | syn::Type::Never(_) | syn::Type::Verbatim(_) => {}

            _ => {}
        }
    }

    fn visit_path_segment(&mut self, segment: &'ast syn::PathSegment) {
        self.visit_path_arguments(&segment.arguments)
    }

    fn visit_path_arguments(&mut self, arguments: &'ast syn::PathArguments) {
        match arguments {
            syn::PathArguments::None => {}
            syn::PathArguments::AngleBracketed(arguments) => {
                for arg in &arguments.args {
                    match arg {
                        syn::GenericArgument::Type(arg) => self.visit_type(arg),
                        syn::GenericArgument::AssocType(arg) => self.visit_type(&arg.ty),
                        syn::GenericArgument::Lifetime(_)
                        | syn::GenericArgument::Const(_)
                        | syn::GenericArgument::AssocConst(_)
                        | syn::GenericArgument::Constraint(_) => {}
                        _ => {}
                    }
                }
            }
            syn::PathArguments::Parenthesized(arguments) => {
                for argument in &arguments.inputs {
                    self.visit_type(argument);
                }
                self.visit_return_type(&arguments.output);
            }
        }
    }

    fn visit_return_type(&mut self, return_type: &'ast syn::ReturnType) {
        match return_type {
            syn::ReturnType::Default => {}
            syn::ReturnType::Type(_, output) => self.visit_type(output),
        }
    }

    fn visit_type_param_bound(&mut self, bound: &'ast syn::TypeParamBound) {
        match bound {
            syn::TypeParamBound::Trait(bound) => self.visit_path(&bound.path),
            syn::TypeParamBound::Lifetime(_)
            | syn::TypeParamBound::PreciseCapture(_)
            | syn::TypeParamBound::Verbatim(_) => {}
            _ => {}
        }
    }

    // Type parameter should not be considered used by a macro path.
    //
    //     struct TypeMacro<T> {
    //         mac: T!(),
    //         marker: PhantomData<T>,
    //     }
    fn visit_macro(&mut self, _mac: &'ast syn::Macro) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_enum_bounds() {
        // All type params should be included in `JsonSchema` trait bounds except `Z`
        let input = parse_quote! {
            #[schemars(rename = "MyEnum<{T}, {U}, {V}, {W}, {X}, {Y}, {{Z}}>")]
            pub enum MyEnum<'a, const LEN: usize, T, U, V, W, X, Y, Z>
            where
                X: Trait,
                Z: OtherTrait
            {
                A,
                B(),
                C(T),
                D(U, (i8, V, bool)),
                E {
                    a: W,
                    b: [&'a Option<Box<<X as Trait>::AssocType::Z>>; LEN],
                    c: Token![Z],
                    d: PhantomData<Z>,
                    #[serde(skip)]
                    e: Z,
                },
                #[serde(skip)]
                F(Z),
            }
        };

        let cont = Container::from_ast(&input).unwrap();

        let (where_clause, relevant_type_params) = find_trait_bounds(&cont);

        assert_eq!(
            where_clause,
            Some(parse_quote!(
                where
                    X: Trait,
                    Z: OtherTrait,
                    T: schemars::JsonSchema,
                    U: schemars::JsonSchema,
                    V: schemars::JsonSchema,
                    W: schemars::JsonSchema,
                    X: schemars::JsonSchema,
                    Y: schemars::JsonSchema
            ))
        );

        let relevant_type_params =
            Vec::from_iter(relevant_type_params.into_iter().map(Ident::to_string));
        assert_eq!(relevant_type_params, vec!["T", "U", "V", "W", "X", "Y"]);
    }
}
