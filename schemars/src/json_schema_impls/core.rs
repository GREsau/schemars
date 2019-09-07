use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::{JsonSchema, Result};
use serde_json::json;

impl<T: JsonSchema> JsonSchema for Option<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        format!("Nullable_{}", T::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Result {
        let mut schema = if gen.settings().option_nullable {
            T::json_schema(gen)?
        } else {
            gen.subschema_for::<T>()?
        };
        if gen.settings().option_add_null_type {
            schema = match schema {
                Schema::Bool(true) => Schema::Bool(true),
                Schema::Bool(false) => <()>::json_schema(gen)?,
                schema => SchemaObject {
                    any_of: Some(vec![schema, <()>::json_schema(gen)?]),
                    ..Default::default()
                }
                .into(),
            }
        }
        if gen.settings().option_nullable {
            let mut deref = gen.get_schema_object(&schema)?;
            deref.extensions.insert("nullable".to_owned(), json!(true));
            schema = Schema::Object(deref);
        };
        Ok(schema)
    }
}

impl<T: ?Sized> JsonSchema for std::marker::PhantomData<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        <()>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Result {
        <()>::json_schema(gen)
    }
}
