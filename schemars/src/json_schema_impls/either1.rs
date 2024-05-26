use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use either1::Either;
use std::borrow::Cow;

impl<L: JsonSchema, R: JsonSchema> JsonSchema for Either<L, R> {
    always_inline!();

    fn schema_name() -> Cow<'static, str> {
        format!("Either_{}_or_{}", L::schema_name(), R::schema_name()).into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("either::Either<{}, {}>", L::schema_id(), R::schema_id()).into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "anyOf": [gen.subschema_for::<L>(), gen.subschema_for::<R>()],
        })
    }
}
