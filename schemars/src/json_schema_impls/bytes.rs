use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use bytes::Bytes;

forward_impl!(JsonSchema for Bytes => Vec<u8>);
