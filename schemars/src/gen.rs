use crate::make_schema::{MakeSchema, SchemaTypeId};
use crate::schema::*;
use std::collections::BTreeMap as Map;
use std::collections::BTreeSet as Set;
use std::iter::FromIterator;

#[derive(Debug, Default)]
pub struct SchemaGenerator {
    names: Set<String>,
    definitions: Map<SchemaTypeId, (String, Schema)>,
}

impl SchemaGenerator {
    pub fn new() -> SchemaGenerator {
        SchemaGenerator {
            ..Default::default()
        }
    }

    pub fn subschema_for<T: ?Sized + MakeSchema>(&mut self) -> Schema {
        if !T::generates_ref_schema() {
            return T::make_schema(self);
        }

        let type_id = T::schema_type_id();
        let name = self
            .definitions
            .get(&type_id)
            .map(|(n, _)| n.clone())
            .unwrap_or_else(|| {
                let name = self.make_unique_name::<T>();
                self.names.insert(name.clone());
                self.insert_new_subschema_for::<T>(type_id, name.clone());
                name
            });
        let reference = format!("#/definitions/{}", name);
        SchemaRef { reference }.into()
    }

    fn insert_new_subschema_for<T: ?Sized + MakeSchema>(
        &mut self,
        type_id: SchemaTypeId,
        name: String,
    ) {
        let dummy = Schema::Bool(false);
        // insert into definitions BEFORE calling make_schema to avoid infinite recursion
        self.definitions.insert(type_id.clone(), (name, dummy));

        let schema = T::make_schema(self);
        self.definitions
            .entry(type_id)
            .and_modify(|(_, s)| *s = schema);
    }

    pub fn definitions(&self) -> Map<String, Schema> {
        Map::from_iter(self.definitions.values().cloned())
    }

    pub fn into_definitions(self) -> Map<String, Schema> {
        Map::from_iter(self.definitions.into_iter().map(|(_, v)| v))
    }

    pub fn root_schema_for<T: ?Sized + MakeSchema>(&mut self) -> Schema {
        let schema = T::make_schema(self);
        if let Schema::Object(mut o) = schema {
            o.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
            o.title = Some(T::schema_name());
            o.definitions.extend(self.definitions());
            return Schema::Object(o);
        }
        schema
    }

    pub fn into_root_schema_for<T: ?Sized + MakeSchema>(mut self) -> Schema {
        let schema = T::make_schema(&mut self);
        if let Schema::Object(mut o) = schema {
            o.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
            o.title = Some(T::schema_name());
            o.definitions.extend(self.into_definitions());
            return Schema::Object(o);
        }
        schema
    }

    fn make_unique_name<T: ?Sized + MakeSchema>(&mut self) -> String {
        let base_name = T::schema_name();
        if self.names.contains(&base_name) {
            for i in 2.. {
                let name = format!("{}{}", base_name, i);
                if !self.names.contains(&name) {
                    return name;
                }
            }
        }
        base_name
    }
}
