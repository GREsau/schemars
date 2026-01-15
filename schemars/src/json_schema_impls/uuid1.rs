use crate::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use alloc::borrow::Cow;
use uuid1::Uuid;
use uuid1::fmt::{Braced, Simple, Hyphenated, Urn};

macro_rules! uuid_variant_schema {
    ($type:ty) => {
        impl JsonSchema for $type {
            inline_schema!();

            fn schema_name() -> Cow<'static, str> {
                "Uuid".into()
            }

            fn schema_id() -> Cow<'static, str> {
                "uuid::Uuid".into()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "type": "string",
                    "format": "uuid",
                })
            }
        }
    };
}

uuid_variant_schema!(Uuid);
uuid_variant_schema!(Simple);
uuid_variant_schema!(Braced);
uuid_variant_schema!(Urn);
uuid_variant_schema!(Hyphenated);
