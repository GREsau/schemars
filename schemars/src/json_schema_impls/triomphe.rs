use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

forward_impl!((<T: ?Sized> JsonSchema for triomphe::Arc<T> where T: JsonSchema) => T);
