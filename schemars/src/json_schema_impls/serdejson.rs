use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use serde_json::{Map, Number, Value};
use std::collections::BTreeMap;

impl JsonSchema for Value {
    no_ref_schema!();

    fn schema_name() -> String {
        "AnyValue".to_owned()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        gen.schema_for_any()
    }
}

impl JsonSchema for Map<String, Value> {
    no_ref_schema!();

    fn schema_name() -> String {
        BTreeMap::<String, Value>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        BTreeMap::<String, Value>::json_schema(gen)
    }
}

impl JsonSchema for Number {
    no_ref_schema!();

    fn schema_name() -> String {
        "Number".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Number.into()),
            ..Default::default()
        }
        .into()
    }
}
