use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use smol_str::SmolStr;

forward_impl!((JsonSchema for SmolStr) => String);
