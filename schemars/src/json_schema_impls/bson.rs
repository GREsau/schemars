use crate::gen::SchemaGenerator;
use crate::schema::{InstanceType, Schema, SchemaObject, StringValidation};
use crate::JsonSchema;
use bson::oid::ObjectId;
use bson::Timestamp;
use std::borrow::Cow;

impl JsonSchema for ObjectId {
    fn schema_name() -> String {
        "ObjectId".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("bson::oid::ObjectId")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation {
                pattern: Some(r"^[0-9a-fA-F]{24}$".to_owned()),
                min_length: Some(24),
                max_length: Some(24),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

impl JsonSchema for Timestamp {
    fn schema_name() -> String {
        "Timestamp".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("bson::Timestamp")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            ..Default::default()
        };
        let obj = schema.object();
        obj.required.insert("time".to_owned());
        obj.properties
            .insert("time".to_owned(), <u32>::json_schema(gen));
        obj.required.insert("increment".to_owned());
        obj.properties
            .insert("increment".to_owned(), <u32>::json_schema(gen));
        schema.into()
    }
}
