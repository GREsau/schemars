use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use sqlx::types::Json;

forward_impl!((<T> JsonSchema for Json<T> where T: JsonSchema) => T);
