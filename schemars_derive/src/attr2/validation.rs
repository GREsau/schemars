use syn::{Attribute, Expr};

use super::AttrCtxt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Format {
    Email,
    Uri,
}

impl Format {
    fn attr_str(self) -> &'static str {
        match self {
            Format::Email => "email",
            Format::Uri => "url",
        }
    }

    fn schema_str(self) -> &'static str {
        match self {
            Format::Email => "email",
            Format::Uri => "uri",
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidationAttrs {
    length_min: Option<Expr>,
    length_max: Option<Expr>,
    length_equal: Option<Expr>,
    range_min: Option<Expr>,
    range_max: Option<Expr>,
    regex: Option<Expr>,
    contains: Option<String>,
    required: bool,
    format: Option<Format>,
    inner: Option<Box<ValidationAttrs>>,
}

impl ValidationAttrs {
    pub fn populate(&mut self, attrs: &[Attribute], schemars_cx: &mut AttrCtxt) {
        self.populate_from_schemars_or_validate(schemars_cx);
        self.populate_from_schemars_or_validate(&mut AttrCtxt::new(
            schemars_cx.inner,
            attrs,
            "validate",
        ));
    }

    fn populate_from_schemars_or_validate(&mut self, cx: &mut AttrCtxt) {
        cx.parse_meta(|meta, meta_name, cx| {
            match meta_name {
                _ => return Some(meta),
            }

            None
        });
    }
}
