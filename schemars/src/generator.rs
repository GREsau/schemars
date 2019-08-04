use crate::make_schema::MakeSchema;
use crate::schema::*;
use core::any::{type_name, TypeId};
use std::collections::HashMap as Map;
use std::collections::HashSet as Set;

#[derive(Debug, Default)]
pub struct SchemaGenerator {
    names: Set<String>,
    definitions: Map<TypeId, (String, Schema)>,
}

impl SchemaGenerator {
    pub fn new() -> SchemaGenerator {
        SchemaGenerator {
            ..Default::default()
        }
    }

    pub fn subschema_for<T: MakeSchema + 'static>(&mut self) -> Schema {
        if !T::generates_ref_schema() {
            return T::make_schema(self);
        }

        let type_id = TypeId::of::<T>();
        // TODO is there a nicer way to do this?
        if !self.definitions.contains_key(&type_id) {
            let name = self.make_unique_name::<T>();
            self.names.insert(name.clone());
            // insert into definitions BEFORE calling make_schema to avoid infinite recursion
            let dummy = Schema::Bool(false);
            self.definitions.insert(type_id, (name.clone(), dummy));

            let schema = T::make_schema(self);
            self.definitions
                .entry(type_id)
                .and_modify(|(_, s)| *s = schema);
        }
        let ref name = self.definitions.get(&type_id).unwrap().0;
        SchemaRef {
            reference: format!("#/definitions/{}", name),
        }
        .into()
    }

    pub fn root_schema_for<T: MakeSchema>(&mut self) -> Schema {
        let schema = T::make_schema(self);
        if let Schema::Object(mut o) = schema {
            o.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
            o.title = Some(T::schema_name());
            for (_, (name, schema)) in self.definitions.iter() {
                o.definitions.insert(name.clone(), schema.clone());
            }
            return Schema::Object(o);
        }
        schema
    }

    pub fn into_root_schema_for<T: MakeSchema>(mut self) -> Schema {
        let schema = T::make_schema(&mut self);
        if let Schema::Object(mut o) = schema {
            o.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
            o.title = Some(T::schema_name());
            for (_, (name, schema)) in self.definitions {
                o.definitions.insert(name, schema);
            }
            return Schema::Object(o);
        }
        schema
    }

    fn make_unique_name<T: MakeSchema>(&mut self) -> String {
        T::schema_name()
        // TODO remove namespace, remove special chars
        // TODO enforce uniqueness
    }
}
