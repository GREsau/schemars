use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::borrow::Cow;

macro_rules! nonzero_unsigned_impl {
    ($type:ty => $primitive:ty) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_id() -> Cow<'static, str> {
                Cow::Borrowed(stringify!($type))
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                let zero_schema: Schema = SchemaObject {
                    const_value: Some(0.into()),
                    ..Default::default()
                }
                .into();
                let mut schema: SchemaObject = <$primitive>::json_schema(gen).into();
                schema.subschemas().not = Some(Box::from(zero_schema));
                schema.into()
            }
        }
    };
}

nonzero_unsigned_impl!(std::num::NonZeroI8 => i8);
nonzero_unsigned_impl!(std::num::NonZeroI16 => i16);
nonzero_unsigned_impl!(std::num::NonZeroI32 => i32);
nonzero_unsigned_impl!(std::num::NonZeroI64 => i64);
nonzero_unsigned_impl!(std::num::NonZeroI128 => i128);
nonzero_unsigned_impl!(std::num::NonZeroIsize => isize);
