use crate::schema::*;
use crate::{JsonSchema, JsonSchemaError, Map, Result};

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

    pub fn subschema_for<T: ?Sized + JsonSchema>(&mut self) -> Result {
        if !T::is_referenceable() {
            return T::json_schema(self);
        }

        let name = T::schema_name();
        let reference = format!("{}{}", self.settings().definitions_path, name);
        if !self.definitions.contains_key(&name) {
            self.insert_new_subschema_for::<T>(name)?;
        }
        Ok(Schema::new_ref(reference))
    }

    fn insert_new_subschema_for<T: ?Sized + JsonSchema>(&mut self, name: String) -> Result<()> {
        let dummy = Schema::Bool(false);
        // insert into definitions BEFORE calling json_schema to avoid infinite recursion
        self.definitions.insert(name.clone(), dummy);

        match T::json_schema(self) {
            Ok(schema) => {
                self.definitions.insert(name, schema);
                Ok(())
            }
            Err(e) => {
                self.definitions.remove(&name);
                Err(e)
            }
        }
    }

    pub fn definitions(&self) -> &Map<String, Schema> {
        &self.definitions
    }

    pub fn into_definitions(self) -> Map<String, Schema> {
        self.definitions
    }

    pub fn root_schema_for<T: ?Sized + JsonSchema>(&mut self) -> Result {
        let schema = T::json_schema(self)?;
        Ok(match schema {
            Schema::Object(mut o) => {
                o.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
                o.title = Some(T::schema_name());
                o.definitions.extend(self.definitions().clone());
                Schema::Object(o)
            }
            schema => schema,
        })
    }

    pub fn into_root_schema_for<T: ?Sized + JsonSchema>(mut self) -> Result {
        let schema = T::json_schema(&mut self)?;
        Ok(match schema {
            Schema::Object(mut o) => {
                o.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
                o.title = Some(T::schema_name());
                o.definitions.extend(self.into_definitions());
                Schema::Object(o)
            }
            schema => schema,
        })
    }

    pub fn dereference_once(&self, schema: Schema) -> Result<Schema> {
        match schema {
            Schema::Object(SchemaObject {
                reference: Some(ref schema_ref),
                ..
            }) => {
                let definitions_path = &self.settings().definitions_path;
                if !schema_ref.starts_with(definitions_path) {
                    return Err(JsonSchemaError::new(
                        "Could not extract referenced schema name.",
                        schema,
                    ));
                }

                let name = &schema_ref[definitions_path.len()..];
                self.definitions.get(name).cloned().ok_or_else(|| {
                    JsonSchemaError::new("Could not find referenced schema.", schema)
                })
            }
            s => Ok(s),
        }
    }

    pub fn dereference(&self, mut schema: Schema) -> Result<Schema> {
        if !schema.is_ref() {
            return Ok(schema);
        }
        for _ in 0..100 {
            schema = self.dereference_once(schema)?;
            if !schema.is_ref() {
                return Ok(schema);
            }
        }
        Err(JsonSchemaError::new(
            "Failed to dereference schema after 100 iterations - references may be cyclic.",
            schema,
        ))
    }
}
