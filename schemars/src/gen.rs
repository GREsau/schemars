use crate::schema::*;
use crate::{JsonSchema, Map};

#[derive(Debug, PartialEq, Clone)]
pub struct SchemaSettings {
    pub option_nullable: bool,
    pub option_add_null_type: bool,
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
        SchemaSettings::new()
    }
}

impl SchemaSettings {
    pub fn new() -> SchemaSettings {
        Self::draft07()
    }

    pub fn draft07() -> SchemaSettings {
        SchemaSettings {
            option_nullable: false,
            option_add_null_type: true,
            bool_schemas: BoolSchemas::Enable,
            definitions_path: "#/definitions/".to_owned(),
        }
    }

    pub fn openapi3() -> SchemaSettings {
        SchemaSettings {
            option_nullable: true,
            option_add_null_type: false,
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
    definitions: Map<String, Schema>,
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
        let schema: Schema = true.into();
        if self.settings().bool_schemas == BoolSchemas::Enable {
            schema
        } else {
            Schema::Object(schema.into())
        }
    }

    pub fn schema_for_none(&self) -> Schema {
        let schema: Schema = false.into();
        if self.settings().bool_schemas == BoolSchemas::Enable {
            schema
        } else {
            Schema::Object(schema.into())
        }
    }

    pub fn subschema_for<T: ?Sized + JsonSchema>(&mut self) -> Schema {
        if !T::is_referenceable() {
            return T::json_schema(self);
        }

        let name = T::schema_name();
        let reference = format!("{}{}", self.settings().definitions_path, name);
        if !self.definitions.contains_key(&name) {
            self.insert_new_subschema_for::<T>(name);
        }
        Schema::new_ref(reference)
    }

    fn insert_new_subschema_for<T: ?Sized + JsonSchema>(&mut self, name: String) {
        let dummy = Schema::Bool(false);
        // insert into definitions BEFORE calling json_schema to avoid infinite recursion
        self.definitions.insert(name.clone(), dummy);
        let schema = T::json_schema(self);
        self.definitions.insert(name, schema);
    }

    pub fn definitions(&self) -> &Map<String, Schema> {
        &self.definitions
    }

    pub fn into_definitions(self) -> Map<String, Schema> {
        self.definitions
    }

    pub fn root_schema_for<T: ?Sized + JsonSchema>(&mut self) -> SchemaObject {
        let mut schema: SchemaObject = T::json_schema(self).into();
        let metadata = schema.metadata();
        metadata.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
        metadata.title = Some(T::schema_name());
        metadata.definitions.extend(self.definitions().clone());
        schema
    }

    pub fn into_root_schema_for<T: ?Sized + JsonSchema>(mut self) -> SchemaObject {
        let mut schema: SchemaObject = T::json_schema(&mut self).into();
        let metadata = schema.metadata();
        metadata.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
        metadata.title = Some(T::schema_name());
        metadata.definitions.extend(self.into_definitions());
        schema
    }

    pub fn dereference<'a>(&'a self, schema: &Schema) -> Option<&'a Schema> {
        match schema {
            Schema::Object(SchemaObject {
                reference: Some(ref schema_ref),
                ..
            }) => {
                let definitions_path = &self.settings().definitions_path;
                let name = if schema_ref.starts_with(definitions_path) {
                    &schema_ref[definitions_path.len()..]
                } else {
                    schema_ref
                };
                self.definitions.get(name)
            }
            _ => None,
        }
    }
}
