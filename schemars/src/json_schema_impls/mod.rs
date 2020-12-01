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

            fn json_schema_for_flatten(gen: &mut SchemaGenerator) -> Schema {
                <$target>::json_schema_for_flatten(gen)
            }

            fn add_schema_as_property(
                gen: &mut SchemaGenerator,
                parent: &mut crate::schema::SchemaObject,
                name: String,
                metadata: Option<crate::schema::Metadata>,
                required: bool,
            ) {
                <$target>::add_schema_as_property(gen, parent, name, metadata, required)
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
#[cfg(feature = "chrono")]
mod chrono;
mod core;
#[cfg(feature = "either")]
mod either;
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
