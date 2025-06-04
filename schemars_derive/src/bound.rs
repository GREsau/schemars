use crate::{
    ast::{Container, Data, Field, Variant},
    attr::WithAttr,
};
use proc_macro2::Span;
use std::collections::BTreeSet;
use syn::{punctuated::Punctuated, Ident};

// This logic is heavily based on serde_derive
// https://github.com/serde-rs/serde/blob/a1ddb18c92f32d64b2ccaf31ddd776e56be34ba2/serde_derive/src/bound.rs#L97

// Returns type params that are actually used in JsonSchema impl, i.e. excluding skipped fields
// and PhantomData
pub fn add_trait_bounds(cont: &mut Container) -> BTreeSet<Ident> {
    let all_type_params =
        BTreeSet::from_iter(cont.generics.type_params().map(|param| &param.ident));

    let all_lifetime_params =
        BTreeSet::from_iter(cont.generics.lifetimes().map(|param| &param.lifetime));

    let type_params_used_in_rename = Vec::from_iter(
        cont.rename_params
            .iter()
            .filter(|i| all_type_params.contains(i)),
    );

    let mut visitor = FindTyParams {
        all_type_params,
        all_lifetime_params,
        relevant_type_params: BTreeSet::new(),
        relevant_field_type_predicates: Vec::new(),
    };

    match &mut cont.data {
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

    // TODO extract types from `relevant_field_type_predicates` for use in auto schema_id?
    let FindTyParams {
        relevant_type_params,
        relevant_field_type_predicates,
        ..
    } = visitor;

    let where_clause = cont.generics.make_where_clause();

    if let Some(bounds) = cont.serde_attrs.ser_bound() {
        where_clause.predicates.extend(bounds.iter().cloned());
    } else {
        where_clause.predicates.extend(
            relevant_field_type_predicates
                .into_iter()
                .map(syn::WherePredicate::Type),
        );

        where_clause
            .predicates
            .extend(type_params_used_in_rename.into_iter().map(|ty| {
                syn::WherePredicate::Type(syn::PredicateType {
                    lifetimes: None,
                    bounded_ty: syn::Type::Path(syn::TypePath {
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: Punctuated::from_iter([syn::PathSegment {
                                ident: ty.clone(),
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

    relevant_type_params
}

fn needs_jsonschema_bound(field: &Field, variant: Option<&Variant>) -> bool {
    if let Some(variant) = variant {
        if variant.serde_attrs.skip_deserializing() && variant.serde_attrs.skip_serializing()
            || matches!(variant.attrs.with, Some(WithAttr::Function(_)))
        {
            return false;
        }
    }
    if field.serde_attrs.skip_deserializing() && field.serde_attrs.skip_serializing()
        || matches!(field.attrs.with, Some(WithAttr::Function(_)))
    {
        return false;
    }

    true
}

struct FindTyParams<'ast> {
    // Set of all generic type parameters on the current struct (A, B, C in
    // the example). Initialized up front.
    all_type_params: BTreeSet<&'ast syn::Ident>,

    all_lifetime_params: BTreeSet<&'ast syn::Lifetime>,

    // Set of generic type parameters used in fields for which filter
    // returns true (A and B in the example). Filled in as the visitor sees
    // them.
    relevant_type_params: BTreeSet<syn::Ident>,

    // Fields whose type makes use of one of the generic type
    // parameters.
    relevant_field_type_predicates: Vec<syn::PredicateType>,
}

#[derive(Default)]
struct FieldInfo {
    uses_type_param: bool,
    uses_lifetime_param: bool,
}

impl<'ast> FindTyParams<'ast> {
    fn visit_field(&mut self, field: &Field) {
        let ty = if let Some(WithAttr::Type(ty)) = &field.attrs.with {
            ty
        } else {
            &field.original.ty
        };
        self.visit_field_type(ty.clone());
    }

    fn visit_field_type(&mut self, mut ty: syn::Type) {
        let mut field_info = FieldInfo {
            uses_type_param: false,
            uses_lifetime_param: false,
        };

        self.visit_type(&mut ty, &mut field_info);

        if field_info.uses_type_param {
            let predicate = syn::PredicateType {
                lifetimes: field_info
                    .uses_lifetime_param
                    .then(|| parse_quote!(for<'_schemars_derive>)),
                bounded_ty: ty,
                colon_token: <Token![:]>::default(),
                bounds: Punctuated::from_iter([syn::TypeParamBound::Trait(syn::TraitBound {
                    paren_token: None,
                    modifier: syn::TraitBoundModifier::None,
                    lifetimes: None,
                    path: parse_quote!(schemars::JsonSchema),
                })]),
            };
            self.relevant_field_type_predicates.push(predicate);
        }
    }

    fn visit_path(&mut self, path: &mut syn::Path, field_info: &mut FieldInfo) {
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
                    self.relevant_type_params.insert(id.clone());
                    field_info.uses_type_param = true;
                }
            }
        }

        for segment in &mut path.segments {
            self.visit_path_segment(segment, field_info);
        }
    }

    fn visit_type(&mut self, ty: &mut syn::Type, field_info: &mut FieldInfo) {
        match ty {
            syn::Type::Array(ty) => self.visit_type(&mut ty.elem, field_info),
            syn::Type::BareFn(ty) => {
                for arg in &mut ty.inputs {
                    self.visit_type(&mut arg.ty, field_info);
                }
                self.visit_return_type(&mut ty.output, field_info);
            }
            syn::Type::Group(ty) => self.visit_type(&mut ty.elem, field_info),
            syn::Type::ImplTrait(ty) => {
                for bound in &mut ty.bounds {
                    self.visit_type_param_bound(bound, field_info);
                }
            }
            syn::Type::Macro(ty) => self.visit_macro(&mut ty.mac, field_info),
            syn::Type::Paren(ty) => self.visit_type(&mut ty.elem, field_info),
            syn::Type::Path(ty) => {
                if let Some(qself) = &mut ty.qself {
                    self.visit_type(&mut qself.ty, field_info);
                }
                self.visit_path(&mut ty.path, field_info);
            }
            syn::Type::Ptr(ty) => self.visit_type(&mut ty.elem, field_info),
            syn::Type::Reference(ty) => {
                if let Some(lifetime) = ty.lifetime.as_mut() {
                    if self.all_lifetime_params.contains(lifetime) {
                        field_info.uses_lifetime_param = true;
                        lifetime.ident = Ident::new("_schemars_derive", Span::call_site())
                    }
                }
                self.visit_type(&mut ty.elem, field_info);
            }
            syn::Type::Slice(ty) => self.visit_type(&mut ty.elem, field_info),
            syn::Type::TraitObject(ty) => {
                for bound in &mut ty.bounds {
                    self.visit_type_param_bound(bound, field_info);
                }
            }
            syn::Type::Tuple(ty) => {
                for elem in &mut ty.elems {
                    self.visit_type(elem, field_info);
                }
            }

            syn::Type::Infer(_) | syn::Type::Never(_) | syn::Type::Verbatim(_) => {}

            _ => {}
        }
    }

    fn visit_path_segment(&mut self, segment: &mut syn::PathSegment, field_info: &mut FieldInfo) {
        self.visit_path_arguments(&mut segment.arguments, field_info)
    }

    fn visit_path_arguments(
        &mut self,
        arguments: &mut syn::PathArguments,
        field_info: &mut FieldInfo,
    ) {
        match arguments {
            syn::PathArguments::None => {}
            syn::PathArguments::AngleBracketed(arguments) => {
                for arg in &mut arguments.args {
                    match arg {
                        syn::GenericArgument::Type(arg) => self.visit_type(arg, field_info),
                        syn::GenericArgument::AssocType(arg) => {
                            self.visit_type(&mut arg.ty, field_info)
                        }
                        syn::GenericArgument::Lifetime(lifetime) => {
                            if self.all_lifetime_params.contains(lifetime) {
                                field_info.uses_lifetime_param = true;
                                lifetime.ident = Ident::new("_schemars_derive", Span::call_site())
                            }
                        }
                        syn::GenericArgument::Const(_)
                        | syn::GenericArgument::AssocConst(_)
                        | syn::GenericArgument::Constraint(_) => {}
                        _ => {}
                    }
                }
            }
            syn::PathArguments::Parenthesized(arguments) => {
                for argument in &mut arguments.inputs {
                    self.visit_type(argument, field_info);
                }
                self.visit_return_type(&mut arguments.output, field_info);
            }
        }
    }

    fn visit_return_type(&mut self, return_type: &mut syn::ReturnType, field_info: &mut FieldInfo) {
        match return_type {
            syn::ReturnType::Default => {}
            syn::ReturnType::Type(_, output) => self.visit_type(output, field_info),
        }
    }

    fn visit_type_param_bound(
        &mut self,
        bound: &mut syn::TypeParamBound,
        field_info: &mut FieldInfo,
    ) {
        match bound {
            syn::TypeParamBound::Trait(bound) => self.visit_path(&mut bound.path, field_info),
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
    fn visit_macro(&mut self, _mac: &mut syn::Macro, _field_info: &mut FieldInfo) {}
}
