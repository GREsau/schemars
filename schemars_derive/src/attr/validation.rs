use super::parse_lit_str;
use proc_macro2::TokenStream;
use syn::ExprLit;
use syn::NestedMeta;
use syn::{Expr, Lit, Meta, MetaNameValue, Path};

#[derive(Debug, Default)]
pub struct ValidationAttrs {
    pub length_min: Option<Expr>,
    pub length_max: Option<Expr>,
    pub length_equal: Option<Expr>,
    pub range_min: Option<Expr>,
    pub range_max: Option<Expr>,
    pub regex: Option<Path>,
    pub contains: Option<String>,
    pub required: bool,
    pub format: Option<&'static str>,
}

impl ValidationAttrs {
    pub fn new(attrs: &[syn::Attribute]) -> Self {
        // TODO allow setting "validate" attributes through #[schemars(...)]
        ValidationAttrs::default().populate(attrs)
    }

    fn populate(mut self, attrs: &[syn::Attribute]) -> Self {
        // TODO don't silently ignore unparseable attributes
        for meta_item in attrs
            .iter()
            .flat_map(|attr| get_meta_items(attr, "validate"))
            .flatten()
        {
            match &meta_item {
                NestedMeta::Meta(Meta::List(meta_list)) if meta_list.path.is_ident("length") => {
                    for nested in meta_list.nested.iter() {
                        match nested {
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("min") => {
                                self.length_min = str_or_num_to_expr(&nv.lit);
                            }
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("max") => {
                                self.length_max = str_or_num_to_expr(&nv.lit);
                            }
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("equal") => {
                                self.length_equal = str_or_num_to_expr(&nv.lit);
                            }
                            _ => {}
                        }
                    }
                }

                NestedMeta::Meta(Meta::List(meta_list)) if meta_list.path.is_ident("range") => {
                    for nested in meta_list.nested.iter() {
                        match nested {
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("min") => {
                                self.range_min = str_or_num_to_expr(&nv.lit);
                            }
                            NestedMeta::Meta(Meta::NameValue(nv)) if nv.path.is_ident("max") => {
                                self.range_max = str_or_num_to_expr(&nv.lit);
                            }
                            _ => {}
                        }
                    }
                }

                NestedMeta::Meta(m)
                    if m.path().is_ident("required") || m.path().is_ident("required_nested") =>
                {
                    self.required = true;
                }

                NestedMeta::Meta(m) if m.path().is_ident("email") => {
                    self.format = Some("email");
                }

                NestedMeta::Meta(m) if m.path().is_ident("url") => {
                    self.format = Some("uri");
                }

                NestedMeta::Meta(m) if m.path().is_ident("phone") => {
                    self.format = Some("phone");
                }

                NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                    path,
                    lit: Lit::Str(regex),
                    ..
                })) if path.is_ident("regex") => self.regex = parse_lit_str(regex).ok(),

                NestedMeta::Meta(Meta::List(meta_list)) if meta_list.path.is_ident("regex") => {
                    self.regex = meta_list.nested.iter().find_map(|x| match x {
                        NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(regex),
                            ..
                        })) if path.is_ident("path") => parse_lit_str(regex).ok(),
                        _ => None,
                    });
                }

                NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                    path,
                    lit: Lit::Str(contains),
                    ..
                })) if path.is_ident("contains") => self.contains = Some(contains.value()),

                NestedMeta::Meta(Meta::List(meta_list)) if meta_list.path.is_ident("contains") => {
                    self.contains = meta_list.nested.iter().find_map(|x| match x {
                        NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(contains),
                            ..
                        })) if path.is_ident("pattern") => Some(contains.value()),
                        _ => None,
                    });
                }

                _ => {}
            }
        }
        self
    }

    pub fn validation_statements(&self, field_name: &str) -> Option<TokenStream> {
        // Assume that the result will be interpolated in a context with the local variable
        // `schema_object` - the SchemaObject for the struct that contains this field.
        let mut array_validation = Vec::new();
        let mut number_validation = Vec::new();
        let mut object_validation = Vec::new();
        let mut string_validation = Vec::new();

        if let Some(length_min) = self
            .length_min
            .as_ref()
            .or_else(|| self.length_equal.as_ref())
        {
            string_validation.push(quote! {
                validation.min_length = Some(#length_min as u32);
            });
            array_validation.push(quote! {
                validation.min_items = Some(#length_min as u32);
            });
        }

        if let Some(length_max) = self
            .length_max
            .as_ref()
            .or_else(|| self.length_equal.as_ref())
        {
            string_validation.push(quote! {
                validation.max_length = Some(#length_max as u32);
            });
            array_validation.push(quote! {
                validation.max_items = Some(#length_max as u32);
            });
        }

        if let Some(range_min) = &self.range_min {
            number_validation.push(quote! {
                validation.minimum = Some(#range_min as f64);
            });
        }

        if let Some(range_max) = &self.range_max {
            number_validation.push(quote! {
                validation.maximum = Some(#range_max as f64);
            });
        }

        if let Some(regex) = &self.regex {
            string_validation.push(quote! {
                validation.pattern = Some(#regex.to_string());
            });
        }

        if let Some(contains) = &self.contains {
            object_validation.push(quote! {
                validation.required.insert(#contains.to_string());
            });

            if self.regex.is_none() {
                let pattern = crate::regex_syntax::escape(contains);
                string_validation.push(quote! {
                    validation.pattern = Some(#pattern.to_string());
                });
            }
        }

        let format = self.format.as_ref().map(|f| {
            quote! {
                prop_schema_object.format = Some(#f.to_string());
            }
        });

        let array_validation = wrap_array_validation(array_validation);
        let number_validation = wrap_number_validation(number_validation);
        let object_validation = wrap_object_validation(object_validation);
        let string_validation = wrap_string_validation(string_validation);

        if array_validation.is_some()
            || number_validation.is_some()
            || object_validation.is_some()
            || string_validation.is_some()
            || format.is_some()
        {
            Some(quote! {
                if let Some(schemars::schema::Schema::Object(prop_schema_object)) = schema_object
                    .object
                    .as_mut()
                    .and_then(|o| o.properties.get_mut(#field_name))
                {
                    #array_validation
                    #number_validation
                    #object_validation
                    #string_validation
                    #format
                }
            })
        } else {
            None
        }
    }
}

fn wrap_array_validation(v: Vec<TokenStream>) -> Option<TokenStream> {
    if v.is_empty() {
        None
    } else {
        Some(quote! {
            if prop_schema_object.has_type(schemars::schema::InstanceType::Array) {
                let validation = prop_schema_object.array();
                #(#v)*
            }
        })
    }
}

fn wrap_number_validation(v: Vec<TokenStream>) -> Option<TokenStream> {
    if v.is_empty() {
        None
    } else {
        Some(quote! {
            if prop_schema_object.has_type(schemars::schema::InstanceType::Integer)
                || prop_schema_object.has_type(schemars::schema::InstanceType::Number) {
                let validation = prop_schema_object.number();
                #(#v)*
            }
        })
    }
}

fn wrap_object_validation(v: Vec<TokenStream>) -> Option<TokenStream> {
    if v.is_empty() {
        None
    } else {
        Some(quote! {
            if prop_schema_object.has_type(schemars::schema::InstanceType::Object) {
                let validation = prop_schema_object.object();
                #(#v)*
            }
        })
    }
}

fn wrap_string_validation(v: Vec<TokenStream>) -> Option<TokenStream> {
    if v.is_empty() {
        None
    } else {
        Some(quote! {
            if prop_schema_object.has_type(schemars::schema::InstanceType::String) {
                let validation = prop_schema_object.string();
                #(#v)*
            }
        })
    }
}

fn get_meta_items(
    attr: &syn::Attribute,
    attr_type: &'static str,
) -> Result<Vec<syn::NestedMeta>, ()> {
    if !attr.path.is_ident(attr_type) {
        return Ok(Vec::new());
    }

    match attr.parse_meta() {
        Ok(Meta::List(meta)) => Ok(meta.nested.into_iter().collect()),
        _ => Err(()),
    }
}

fn str_or_num_to_expr(lit: &Lit) -> Option<Expr> {
    match lit {
        Lit::Str(s) => parse_lit_str::<syn::ExprPath>(s).ok().map(Expr::Path),
        Lit::Int(_) | Lit::Float(_) => Some(Expr::Lit(ExprLit {
            attrs: Vec::new(),
            lit: lit.clone(),
        })),
        _ => None,
    }
}
