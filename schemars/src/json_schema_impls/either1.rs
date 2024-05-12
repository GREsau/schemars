use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use either1::Either;
use std::borrow::Cow;

impl<L: JsonSchema, R: JsonSchema> JsonSchema for Either<L, R> {
    no_ref_schema!();

    fn schema_name() -> String {
        format!("Either_{}_or_{}", L::schema_name(), R::schema_name())
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Owned(format!(
            "either::Either<{}, {}>",
            L::schema_id(),
            R::schema_id()
        ))
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "anyOf": [gen.subschema_for::<L>(), gen.subschema_for::<R>()],
        })
    }
}
