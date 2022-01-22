macro_rules! no_ref_schema {
    () => {
        fn is_referenceable() -> bool {
            false
        }
    };
}

macro_rules! forward_impl {
    (($($impl:tt)+) => $target:ty) => {
        impl $($impl)+ {
            fn is_referenceable() -> bool {
                <$target>::is_referenceable()
            }

            fn schema_name() -> String {
                <$target>::schema_name()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                <$target>::json_schema(gen)
            }

            fn _schemars_private_non_optional_json_schema(gen: &mut SchemaGenerator) -> Schema {
                <$target>::_schemars_private_non_optional_json_schema(gen)
            }

            fn _schemars_private_is_option() -> bool {
                <$target>::_schemars_private_is_option()
            }
        }
    };
    ($ty:ty => $target:ty) => {
        forward_impl!((JsonSchema for $ty) => $target);
    };
}

mod array;
#[cfg(feature = "arrayvec")]
mod arrayvec;
#[cfg(std_atomic)]
mod atomic;
#[cfg(feature = "bytes")]
mod bytes;
#[cfg(feature = "chrono")]
mod chrono;
mod core;
#[cfg(any(feature = "rust_decimal", feature="bigdecimal"))]
mod decimal;
#[cfg(feature = "either")]
mod either;
#[cfg(feature = "enumset")]
mod enumset;
mod ffi;
#[cfg(feature = "indexmap")]
mod indexmap;
mod maps;
mod nonzero_signed;
mod nonzero_unsigned;
mod primitives;
mod sequences;
mod serdejson;
#[cfg(feature = "smallvec")]
mod smallvec;
mod time;
mod tuple;
#[cfg(feature = "url")]
mod url;
#[cfg(feature = "uuid")]
mod uuid;
mod wrapper;
