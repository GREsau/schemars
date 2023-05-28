use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use netidx_core::path::Path;

forward_impl!((JsonSchema for Path) => String);
