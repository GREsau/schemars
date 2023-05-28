use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use enumflags2::{BitFlags, _internal::RawBitFlags};

forward_impl!((<T> JsonSchema for BitFlags<T> where T: RawBitFlags) => u64);
