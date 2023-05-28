use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use netidx_core::{path::Path, chars::Chars};

forward_impl!((JsonSchema for Path) => String);
forward_impl!((JsonSchema for Chars) => String);
