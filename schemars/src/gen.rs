use crate::schema::*;
use crate::{JsonSchema, Map};

/// Settings to customize how a Schema is generated.
#[derive(Debug, PartialEq, Clone)]
pub struct SchemaSettings {
    /// If `true`, schemas for [`Option<T>`](Option) will include a `nullable` property.
    ///
    /// This is not part of the JSON Schema spec, but is used in Swagger/OpenAPI schemas.
    ///
    /// Defaults to `false`.
    pub option_nullable: bool,
    /// If `true`, schemas for [`Option<T>`](Option) will have `null` added to their [`type`](../schema/struct.SchemaObject.html#structfield.instance_type).
    ///
    /// Defaults to `true`.
    pub option_add_null_type: bool,
    /// Controls whether trivial [`Bool`](../schema/enum.Schema.html#variant.Bool) schemas may be generated.
    ///
    /// Defaults to [`BoolSchemas::Enabled`].
    pub bool_schemas: BoolSchemas,
    /// A JSON pointer to the expected location of referenceable subschemas within the resulting root schema.
    ///
    /// Defaults to `"#/definitions/"`.
    pub definitions_path: String,
}

/// Controls whether trivial [`Bool`](../schema/enum.Schema.html#variant.Bool) schemas may be generated.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BoolSchemas {
    /// `Bool` schemas may be used.
    Enabled,
    /// `Bool` schemas may only be used in a schema's [`additionalProperties`](../sschema/struct.ObjectValidation.html#structfield.additional_properties) field.
    AdditionalPropertiesOnly,
    /// `Bool` schemas will never be used.
    Disabled,
}

impl Default for SchemaSettings {
    fn default() -> SchemaSettings {
        SchemaSettings::new()
    }
}

impl SchemaSettings {
    /// Creates `SchemaSettings` that conform to [JSON Schema Draft 7](https://json-schema.org/specification-links.html#draft-7).
    pub fn new() -> SchemaSettings {
        Self::draft07()
    }

    /// Creates `SchemaSettings` that conform to [JSON Schema Draft 7](https://json-schema.org/specification-links.html#draft-7).
    pub fn draft07() -> SchemaSettings {
        SchemaSettings {
            option_nullable: false,
            option_add_null_type: true,
            bool_schemas: BoolSchemas::Enabled,
            definitions_path: "#/definitions/".to_owned(),
        }
    }

    /// Creates `SchemaSettings` that conform to [OpenAPI 3](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.0.md#schemaObject).
    pub fn openapi3() -> SchemaSettings {
        SchemaSettings {
            option_nullable: true,
            option_add_null_type: false,
            bool_schemas: BoolSchemas::AdditionalPropertiesOnly,
            definitions_path: "#/components/schemas/".to_owned(),
        }
    }

    /// Creates a new [`SchemaGenerator`] using these settings.
    pub fn into_generator(self) -> SchemaGenerator {
        SchemaGenerator::new(self)
    }
}

/// The main type used to generate JSON Schemas.
///
/// # Example
/// ```
/// use schemars::{JsonSchema, gen::SchemaGenerator};
///
/// #[derive(JsonSchema)]
/// struct MyStruct {
///     foo: i32,
/// }
///
/// let gen = SchemaGenerator::default();
/// let schema = gen.into_root_schema_for::<MyStruct>();
/// ```
#[derive(Debug, Default, Clone)]
pub struct SchemaGenerator {
    settings: SchemaSettings,
    definitions: Map<String, Schema>,
}

impl SchemaGenerator {
    /// Creates a new `SchemaGenerator` using the given settings.
    pub fn new(settings: SchemaSettings) -> SchemaGenerator {
        SchemaGenerator {
            settings,
            ..Default::default()
        }
    }

    /// Borrows the [`SchemaSettings`] being used by this `SchemaGenerator`.
    ///
    /// # Example
    /// ```
    /// use schemars::gen::SchemaGenerator;
    ///
    /// let gen = SchemaGenerator::default();
    /// let settings = gen.settings();
    ///
    /// assert_eq!(settings.option_add_null_type, true);
    /// ```
    pub fn settings(&self) -> &SchemaSettings {
        &self.settings
    }

    /// Returns a `Schema` that matches everything, such as the empty schema `{}`.
    ///
    /// The exact value returned depends on this generator's [`BoolSchemas`](struct.SchemaSettings.html#structfield.bool_schemas) setting.
    pub fn schema_for_any(&self) -> Schema {
        let schema: Schema = true.into();
        if self.settings().bool_schemas == BoolSchemas::Enabled {
            schema
        } else {
            Schema::Object(schema.into())
        }
    }

    /// Returns a `Schema` that matches nothing, such as the schema `{ "not":{} }`.
    ///
    /// The exact value returned depends on this generator's [`BoolSchemas`](struct.SchemaSettings.html#structfield.bool_schemas) setting.
    pub fn schema_for_none(&self) -> Schema {
        let schema: Schema = false.into();
        if self.settings().bool_schemas == BoolSchemas::Enabled {
            schema
        } else {
            Schema::Object(schema.into())
        }
    }

    /// Generates a JSON Schema for the type `T`, and returns either the schema itself or a `$ref` schema referencing `T`'s schema.
    ///
    /// If `T` is [referenceable](JsonSchema::is_referenceable), this will add `T`'s schema to this generator's definitions, and
    /// return a `$ref` schema referencing that schema. Otherwise, this method behaves identically to [`JsonSchema::json_schema`].
    ///
    /// If `T`'s schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// add them to the `SchemaGenerator`'s schema definitions.
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

    /// Returns the collection of all [referenceable](JsonSchema::is_referenceable) schemas that have been generated.
    ///
    /// The keys of the returned `Map` are the [schema names](JsonSchema::schema_name), and the values are the schemas
    /// themselves.
    pub fn definitions(&self) -> &Map<String, Schema> {
        &self.definitions
    }

    /// Consumes `self` and returns the collection of all [referenceable](JsonSchema::is_referenceable) schemas that have been generated.
    ///
    /// The keys of the returned `Map` are the [schema names](JsonSchema::schema_name), and the values are the schemas
    /// themselves.
    pub fn into_definitions(self) -> Map<String, Schema> {
        self.definitions
    }

    /// Generates a root JSON Schema for the type `T`.
    ///
    /// If `T`'s schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// add them to the `SchemaGenerator`'s schema definitions and include them in the returned `SchemaObject`'s
    /// [`definitions`](../schema/struct.Metadata.html#structfield.definitions)
    pub fn root_schema_for<T: ?Sized + JsonSchema>(&mut self) -> SchemaObject {
        let mut schema: SchemaObject = T::json_schema(self).into();
        let metadata = schema.metadata();
        metadata.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
        metadata.title = Some(T::schema_name());
        metadata.definitions.extend(self.definitions().clone());
        schema
    }

    /// Consumes `self` and generates a root JSON Schema for the type `T`.
    ///
    /// If `T`'s schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// include them in the returned `SchemaObject`'s [`definitions`](../schema/struct.Metadata.html#structfield.definitions)
    pub fn into_root_schema_for<T: ?Sized + JsonSchema>(mut self) -> SchemaObject {
        let mut schema: SchemaObject = T::json_schema(&mut self).into();
        let metadata = schema.metadata();
        metadata.schema = Some("http://json-schema.org/draft-07/schema#".to_owned());
        metadata.title = Some(T::schema_name());
        metadata.definitions.extend(self.into_definitions());
        schema
    }

    /// Attemps to find the schema that the given `schema` is referencing.
    ///
    /// If the given `schema` has a [`$ref`](../schema/struct.SchemaObject.html#structfield.reference) property which refers
    /// to another schema in `self`'s schema definitions, the referenced schema will be returned. Otherwise, returns `None`.
    ///
    /// # Example
    /// ```
    /// use schemars::{JsonSchema, gen::SchemaGenerator};
    ///
    /// #[derive(JsonSchema)]
    /// struct MyStruct {
    ///     foo: i32,
    /// }
    ///
    /// let mut gen = SchemaGenerator::default();
    /// let ref_schema = gen.subschema_for::<MyStruct>();
    ///
    /// assert!(ref_schema.is_ref());
    ///
    /// let dereferenced = gen.dereference(&ref_schema);
    ///
    /// assert!(dereferenced.is_some());
    /// assert!(!dereferenced.unwrap().is_ref());
    /// ```
    pub fn dereference<'a>(&'a self, schema: &Schema) -> Option<&'a Schema> {
        match schema {
            Schema::Object(SchemaObject {
                reference: Some(ref schema_ref),
                ..
            }) => {
                let definitions_path = &self.settings().definitions_path;
                if schema_ref.starts_with(definitions_path) {
                    let name = &schema_ref[definitions_path.len()..];
                    self.definitions.get(name)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
