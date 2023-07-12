use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use crate::{Map, Set};

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            K: JsonSchema,
            V: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                format!("Map_of_{}", V::schema_name())
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                let k_subschema = gen.subschema_for::<K>();
                let v_subschema = gen.subschema_for::<V>();
                // if the map's key is a reference to another schema, and that schema is an
                // enum, our final schema should require that the map has key values
                // that are one of the enum values
                if let Some(Schema::Object(schema_object)) = gen.dereference(&k_subschema) {
                    let mut schemas: Vec<Schema> = vec![];
                    if let Some(values) = &schema_object.enum_values {
                        for value in values {
                            let schema = SchemaObject {
                                instance_type: Some(InstanceType::Object.into()),
                                object: Some(Box::new(ObjectValidation {
                                    required: Set::from([value.to_string()]),
                                    properties: Map::from([(value.to_string(), v_subschema.clone())]),
                                    ..Default::default()
                                })),
                                ..Default::default()
                            };
                            schemas.push(schema.into());
                        }
                        let mut schema = SchemaObject::default();
                        schema.subschemas().one_of = Some(schemas);
                        return schema.into();
                    }
                }
                // if the key's schema is not a reference, or if the dereferenced key is not an enum,
                // we can only enforce map values and not the map keys
                SchemaObject {
                    instance_type: Some(InstanceType::Object.into()),
                    object: Some(Box::new(ObjectValidation {
                        additional_properties: Some(Box::new(v_subschema)),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into()
            }
        }
    };
}

map_impl!(<K, V> JsonSchema for std::collections::BTreeMap<K, V>);
map_impl!(<K, V, H> JsonSchema for std::collections::HashMap<K, V, H>);
