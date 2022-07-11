use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use tinyvec::{Array, ArrayVec, TinyVec};

forward_impl!((<A: Array> JsonSchema for TinyVec<A> where A::Item: JsonSchema) => Vec<A::Item>);
forward_impl!((<A: Array> JsonSchema for ArrayVec<A> where A::Item: JsonSchema) => Vec<A::Item>);
