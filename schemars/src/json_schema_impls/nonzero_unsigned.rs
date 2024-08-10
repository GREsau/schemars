use crate::gen::SchemaGenerator;
use crate::JsonSchema;
use crate::Schema;
use std::borrow::Cow;
use std::num::*;

macro_rules! nonzero_unsigned_impl {
    ($type:ty => $primitive:ty) => {
        impl JsonSchema for $type {
            always_inline!();

            fn schema_name() -> Cow<'static, str> {
                stringify!($type).into()
            }

            fn schema_id() -> Cow<'static, str> {
                stringify!(std::num::$type).into()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                let mut schema = <$primitive>::json_schema(gen);
                let object = schema.ensure_object();
                object.insert("minimum".to_owned(), 1.into());
                schema
            }
        }
    };
}

nonzero_unsigned_impl!(NonZeroU8 => u8);
nonzero_unsigned_impl!(NonZeroU16 => u16);
nonzero_unsigned_impl!(NonZeroU32 => u32);
nonzero_unsigned_impl!(NonZeroU64 => u64);
nonzero_unsigned_impl!(NonZeroU128 => u128);
nonzero_unsigned_impl!(NonZeroUsize => usize);
