macro_rules! always_inline {
    () => {
        fn always_inline_schema() -> bool {
            true
        }
    };
}

macro_rules! forward_impl {
    (($($impl:tt)+) => $target:ty) => {
        impl $($impl)+ {
            fn always_inline_schema() -> bool {
                <$target>::always_inline_schema()
            }

            fn schema_name() -> alloc::borrow::Cow<'static, str> {
                <$target>::schema_name()
            }

            fn schema_id() -> alloc::borrow::Cow<'static, str> {
                <$target>::schema_id()
            }

            fn json_schema(gen: &mut $crate::gen::SchemaGenerator) -> $crate::Schema {
                <$target>::json_schema(gen)
            }

            fn _schemars_private_non_optional_json_schema(gen: &mut $crate::gen::SchemaGenerator) -> $crate::Schema {
                <$target>::_schemars_private_non_optional_json_schema(gen)
            }

            fn _schemars_private_is_option() -> bool {
                <$target>::_schemars_private_is_option()
            }
        }
    };
    ($ty:ty => $target:ty) => {
        forward_impl!(($crate::JsonSchema for $ty) => $target);
    };
}

mod array;
mod core;
mod maps;
mod nonzero_signed;
mod nonzero_unsigned;
mod primitives;
mod sequences;
mod serdejson;
mod std_time;
mod tuple;
mod wrapper;

#[cfg(target_has_atomic)]
mod atomic;

#[cfg(feature = "std")]
mod ffi;

#[cfg(feature = "arrayvec07")]
mod arrayvec07;

#[cfg(feature = "bytes1")]
mod bytes1 {
    forward_impl!(bytes1::Bytes => alloc::vec::Vec<u8>);
    forward_impl!(bytes1::BytesMut => alloc::vec::Vec<u8>);
}

#[cfg(feature = "chrono04")]
mod chrono04;

#[cfg(any(feature = "rust_decimal1", feature = "bigdecimal04"))]
mod decimal;

#[cfg(feature = "either1")]
mod either1;

#[cfg(feature = "enumset1")]
forward_impl!((<T: enumset1::EnumSetType + crate::JsonSchema> crate::JsonSchema for enumset1::EnumSet<T>) => alloc::collections::BTreeSet<T>);

#[cfg(feature = "indexmap2")]
mod indexmap2;

#[cfg(feature = "ipnetwork020")]
mod ipnetwork020;

#[cfg(feature = "semver1")]
mod semver1;

#[cfg(feature = "smallvec1")]
forward_impl!((<A: smallvec1::Array> crate::JsonSchema for smallvec1::SmallVec<A> where A::Item: crate::JsonSchema) => alloc::vec::Vec<A::Item>);

#[cfg(feature = "smol_str02")]
forward_impl!(smol_str02::SmolStr => alloc::string::String);

#[cfg(feature = "url2")]
mod url2;

#[cfg(feature = "uuid1")]
mod uuid1;
