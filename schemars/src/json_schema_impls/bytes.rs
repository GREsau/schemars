use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use bytes::Bytes;

forward_impl!((Bytes JsonSchema) => Vec<A::Item>);
