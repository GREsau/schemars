use crate::make_schema::{MakeSchema, SchemaTypeId};
use crate::schema::*;
use std::collections::BTreeMap as Map;
use std::collections::BTreeSet as Set;
use std::iter::FromIterator;

#[derive(Debug, PartialEq, Clone)]
pub struct SchemaSettings {
    pub option_nullable: bool,
    pub option_any_of_null: bool,
    pub bool_schemas: BoolSchemas,
    pub definitions_path: String,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BoolSchemas {
    Enable,
    AdditionalPropertiesOnly,
    Disable,
}

impl Default for SchemaSettings {
    fn default() -> SchemaSettings {
        SchemaSettings {
            option_nullable: false,
            option_any_of_null: true,
            bool_schemas: BoolSchemas::Enable,
            definitions_path: "#/definitions/".to_owned(),
        }
    }
}

impl SchemaSettings {
    pub fn new() -> SchemaSettings {
        SchemaSettings {
            ..Default::default()
        }
    }
    pub fn openapi3() -> SchemaSettings {
        SchemaSettings {
            option_nullable: true,
            option_any_of_null: false,
            bool_schemas: BoolSchemas::AdditionalPropertiesOnly,
            definitions_path: "#/components/schemas/".to_owned(),
        }
    }

    pub fn into_generator(self) -> SchemaGenerator {
        SchemaGenerator::new(self)
    }
}

#[derive(Debug, Default, Clone)]
pub struct SchemaGenerator {
    settings: SchemaSettings,
    names: Set<String>,
    definitions: Map<SchemaTypeId, (String, Schema)>,
}

impl SchemaGenerator {
    pub fn new(settings: SchemaSettings) -> SchemaGenerator {
        SchemaGenerator {
            settings,
            ..Default::default()
        }
    }

    pub fn settings(&self) -> &SchemaSettings {
        &self.settings
    }

    pub fn schema_for_any(&self) -> Schema {
        if self.settings().bool_schemas == BoolSchemas::Enable {
            true.into()
        } else {
            Schema::Object(Default::default())
        }
    }

    pub fn schema_for_none(&self) -> Schema {
        if self.settings().bool_schemas == BoolSchemas::Enable {
            false.into()
        } else {
            Schema::Object(SchemaObject {
                not: Some(Schema::Object(Default::default()).into()),
                ..Default::default()
            })
        }
    }

    pub fn subschema_for<T: ?Sized + MakeSchema>(&mut self) -> Schema {
        if !T::is_referenceable() {
            return T::make_schema(self);
        }

        let type_id = T::schema_type_id();
        let name = self
            .definitions
            .get(&type_id)
            .map(|(n, _)| n.clone())
            .unwrap_or_else(|| {
                let name = self.make_unique_name::<T>();
                self.insert_new_subschema_for::<T>(type_id, name.clone());
                name
            });
        let reference = format!("{}{}", self.settings().definitions_path, name);
        SchemaRef { reference }.into()
    }

    fn insert_new_subschema_for<T: ?Sized + MakeSchema>(
        &mut self,
        type_id: SchemaTypeId,
        name: String,
    ) {
        self.names.insert(name.clone());
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

    pub(crate) fn try_get_schema_object<'a>(
        &'a self,
        mut schema: &'a Schema,
    ) -> Option<SchemaObject> {
        loop {
            match schema {
                Schema::Object(o) => return Some(o.clone()),
                Schema::Bool(true) => return Some(Default::default()),
                Schema::Bool(false) => {
                    return Some(SchemaObject {
                        not: Some(Schema::Bool(true).into()),
                        ..Default::default()
                    })
                }
                Schema::Ref(r) => {
                    let definitions_path_len = self.settings().definitions_path.len();
                    let name = r.reference.get(definitions_path_len..)?;
                    // FIXME this is pretty inefficient
                    schema = self
                        .definitions
                        .values()
                        .filter(|(n, _)| n == name)
                        .map(|(_, s)| s)
                        .next()?;
                }
            }
        }
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
