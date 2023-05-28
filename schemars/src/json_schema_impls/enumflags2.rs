use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use enumflags2::BitFlags;

forward_impl!((<T> JsonSchema for BitFlags<T>) => u64);
