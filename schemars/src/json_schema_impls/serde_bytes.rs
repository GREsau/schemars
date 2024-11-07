use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use serde_bytes::ByteBuf;

forward_impl!((JsonSchema for ByteBuf) => alloc::vec::Vec<u8>);
// Because Bytes is a wrapper around [u8] which is not `Sized`
// I couldn't get it through the testsuite to check if this actually works.
//
// use serde_bytes::Bytes;
// forward_impl!((JsonSchema for Bytes) => Vec<u8>);
