use syn::{Attribute, Expr, Meta};

use super::{
    parse_meta::{parse_length_or_range, parse_nested_meta, require_path_only, LengthOrRange},
    AttrCtxt,
};

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

    fn from_attr_str(s: &str) -> Self {
        match s {
            "email" => Format::Email,
            "url" => Format::Uri,
            _ => panic!("Invalid format attr string `{}`. This is a bug in schemars, please raise an issue!", s),
        }
    }
}

#[derive(Debug, Default)]
pub struct ValidationAttrs {
    pub length: Option<LengthOrRange>,
    pub range: Option<LengthOrRange>,
    pub regex: Option<Expr>,
    pub contains: Option<String>,
    pub required: bool,
    pub format: Option<Format>,
    pub inner: Option<Box<ValidationAttrs>>,
}

impl ValidationAttrs {
    pub fn populate(&mut self, attrs: &[Attribute], schemars_cx: &mut AttrCtxt) {
        self.process_attr(schemars_cx);
        self.process_attr(&mut AttrCtxt::new(schemars_cx.inner, attrs, "validate"));
    }

    fn process_attr(&mut self, cx: &mut AttrCtxt) {
        cx.parse_meta(|m, n, c| self.process_meta(m, n, c));
    }

    fn process_meta(&mut self, meta: Meta, meta_name: &str, cx: &AttrCtxt) -> Option<Meta> {
        match meta_name {
            "length" => match self.length {
                Some(_) => cx.duplicate_error(&meta),
                None => self.length = parse_length_or_range(meta, cx).ok(),
            },

            "range" => match self.range {
                Some(_) => cx.duplicate_error(&meta),
                None => self.range = parse_length_or_range(meta, cx).ok(),
            },

            "email" | "url" => self.handle_format(meta, meta_name, cx),

            "required" => {
                if self.required {
                    cx.duplicate_error(&meta);
                } else if require_path_only(meta, cx).is_ok() {
                    self.required = true;
                }
            }

            "regex" => todo!(),
            "contains" => todo!(),

            "inner" => {
                if let Ok(nested_meta) = parse_nested_meta(meta, cx) {
                    let inner = self
                        .inner
                        .get_or_insert_with(|| Box::new(ValidationAttrs::default()));
                    let mut inner_cx = cx.new_nested_meta(nested_meta.into_iter().collect());
                    inner.process_attr(&mut inner_cx);
                }
            }

            _ => return Some(meta),
        }

        None
    }

    fn handle_format(&mut self, meta: Meta, meta_name: &str, cx: &AttrCtxt) {
        match &self.format {
            Some(f) if f.attr_str() == meta_name => cx.duplicate_error(&meta),
            Some(f) => cx.mutual_exclusive_error(&meta, f.attr_str()),
            None => {
                if require_path_only(meta, cx).is_ok() {
                    self.format = Some(Format::from_attr_str(meta_name))
                }
            }
        }
    }
}
