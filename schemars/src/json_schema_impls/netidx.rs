use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use netidx_core::{path::Path, chars::Chars, pool::{Poolable, Pooled}};

forward_impl!((JsonSchema for Path) => String);
forward_impl!((JsonSchema for Chars) => String);
forward_impl!((<T> JsonSchema for Pooled<T> where T: Poolable + Sync + Send + JsonSchema + 'static) => T);
