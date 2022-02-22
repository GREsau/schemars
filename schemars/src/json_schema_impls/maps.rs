use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::borrow::Cow;
use std::collections::*;

impl<K, V: JsonSchema> JsonSchema for BTreeMap<K, V> {
    no_ref_schema!();

    fn schema_id() -> Cow<'static, str> {
        Cow::Owned(format!(
            "std::collections::BTreeMap<str, {}>",
            V::schema_id()
        ))
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let subschema = gen.subschema_for::<V>();
        SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                additional_properties: Some(Box::new(subschema)),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

forward_impl!((<K, V: JsonSchema, H> JsonSchema for HashMap<K, V, H>) => BTreeMap<K, V>);
