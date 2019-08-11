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
        SchemaSettings {
            option_nullable: false,
            option_add_null_type: true,
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

    pub fn subschema_for<T: ?Sized + JsonSchema>(&mut self) -> Result {
        if !T::is_referenceable() {
            return T::json_schema(self);
        }

        let name = T::schema_name();
        if !self.definitions.contains_key(&name) {
            self.insert_new_subschema_for::<T>(name.clone())?;
        }
        let reference = format!("{}{}", self.settings().definitions_path, name);
        Ok(Ref { reference }.into())
    }

    fn insert_new_subschema_for<T: ?Sized + JsonSchema>(&mut self, name: String) -> Result<()> {
        let dummy = Schema::Bool(false);
        // insert into definitions BEFORE calling json_schema to avoid infinite recursion
        self.definitions.insert(name.clone(), dummy);

        match T::json_schema(self) {
            Ok(schema) => {
                self.definitions.insert(name.clone(), schema);
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

    pub(crate) fn get_schema_object<'a>(&'a self, mut schema: &'a Schema) -> Result<SchemaObject> {
        loop {
            match schema {
                Schema::Object(o) => return Ok(o.clone()),
                Schema::Bool(true) => return Ok(Default::default()),
                Schema::Bool(false) => {
                    return Ok(SchemaObject {
                        not: Some(Schema::Bool(true).into()),
                        ..Default::default()
                    })
                }
                Schema::Ref(r) => {
                    let definitions_path_len = self.settings().definitions_path.len();
                    let name = r.reference.get(definitions_path_len..).ok_or_else(|| {
                        JsonSchemaError::new(
                            "Could not extract referenced schema name.",
                            Schema::Ref(r.clone()),
                        )
                    })?;

                    schema = self.definitions.get(name).ok_or_else(|| {
                        JsonSchemaError::new(
                            "Could not find referenced schema.",
                            Schema::Ref(r.clone()),
                        )
                    })?;

                    match schema {
                        Schema::Ref(r2) if r2 == r => {
                            return Err(JsonSchemaError::new(
                                "Schema is referencing itself.",
                                schema.clone(),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
