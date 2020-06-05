use crate::flatten::Merge;
use crate::schema::*;
use crate::{visit::*, JsonSchema, Map};
use std::sync::Arc;

/// Settings to customize how Schemas are generated.
///
/// The default settings currently conform to [JSON Schema Draft 7](https://json-schema.org/specification-links.html#draft-7), but this is liable to change in a future version of Schemars if support for other JSON Schema versions is added.
/// If you require your generated schemas to conform to draft 7, consider using the [`draft07`](#method.draft07) method.
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
    /// A JSON pointer to the expected location of referenceable subschemas within the resulting root schema.
    ///
    /// Defaults to `"#/definitions/"`.
    pub definitions_path: String,
    /// The URI of the meta-schema describing the structure of the generated schemas.
    ///
    /// Defaults to `"http://json-schema.org/draft-07/schema#"`.
    pub meta_schema: Option<String>,
    /// TODO document
    pub visitors: Visitors,
    _hidden: (),
}

impl Default for SchemaSettings {
    fn default() -> SchemaSettings {
        SchemaSettings::draft07()
    }
}

impl SchemaSettings {
    /// Creates `SchemaSettings` that conform to [JSON Schema Draft 7](https://json-schema.org/specification-links.html#draft-7).
    pub fn draft07() -> SchemaSettings {
        SchemaSettings {
            option_nullable: false,
            option_add_null_type: true,
            definitions_path: "#/definitions/".to_owned(),
            meta_schema: Some("http://json-schema.org/draft-07/schema#".to_owned()),
            visitors: Visitors(vec![Arc::new(RemoveRefSiblings)]),
            _hidden: (),
        }
    }

    /// Creates `SchemaSettings` that conform to [JSON Schema 2019-09](https://json-schema.org/specification-links.html#2019-09-formerly-known-as-draft-8).
    pub fn draft2019_09() -> SchemaSettings {
        SchemaSettings {
            option_nullable: false,
            option_add_null_type: true,
            definitions_path: "#/definitions/".to_owned(),
            meta_schema: Some("https://json-schema.org/draft/2019-09/schema".to_owned()),
            visitors: Visitors::default(),
            _hidden: (),
        }
    }

    /// Creates `SchemaSettings` that conform to [OpenAPI 3.0](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.0.md#schemaObject).
    pub fn openapi3() -> SchemaSettings {
        SchemaSettings {
            option_nullable: true,
            option_add_null_type: false,
            definitions_path: "#/components/schemas/".to_owned(),
            meta_schema: Some(
                "https://spec.openapis.org/oas/3.0/schema/2019-04-02#/definitions/Schema"
                    .to_owned(),
            ),
            visitors: Visitors(vec![
                Arc::new(RemoveRefSiblings),
                Arc::new(ReplaceBoolSchemas {
                    skip_additional_properties: true,
                }),
            ]),
            _hidden: (),
        }
    }

    /// Modifies the `SchemaSettings` by calling the given function.
    ///
    /// # Example
    /// ```
    /// use schemars::gen::{SchemaGenerator, SchemaSettings};
    ///
    /// let settings = SchemaSettings::default().with(|s| {
    ///     s.option_nullable = true;
    ///     s.option_add_null_type = false;
    /// });
    /// let gen = settings.into_generator();
    /// ```
    pub fn with(mut self, configure_fn: impl FnOnce(&mut Self)) -> Self {
        configure_fn(&mut self);
        self
    }

    /// TODO document
    pub fn with_visitor(mut self, visitor: impl Visitor) -> Self {
        self.visitors.0.push(Arc::new(visitor));
        self
    }

    /// Creates a new [`SchemaGenerator`] using these settings.
    pub fn into_generator(self) -> SchemaGenerator {
        SchemaGenerator::new(self)
    }
}

/// TODO document
#[derive(Debug, Clone, Default)]
pub struct Visitors(Vec<Arc<dyn Visitor>>);

impl PartialEq for Visitors {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(a, b)| Arc::ptr_eq(a, b))
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

impl From<SchemaSettings> for SchemaGenerator {
    fn from(settings: SchemaSettings) -> Self {
        settings.into_generator()
    }
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

    #[deprecated = "This method no longer has any effect."]
    pub fn make_extensible(&self, _schema: &mut SchemaObject) {}

    #[deprecated = "Use `Schema::Bool(true)` instead"]
    pub fn schema_for_any(&self) -> Schema {
        Schema::Bool(true)
    }

    #[deprecated = "Use `Schema::Bool(false)` instead"]
    pub fn schema_for_none(&self) -> Schema {
        Schema::Bool(false)
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

    /// Borrows the collection of all [referenceable](JsonSchema::is_referenceable) schemas that have been generated.
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
    pub fn root_schema_for<T: ?Sized + JsonSchema>(&mut self) -> RootSchema {
        let mut schema = T::json_schema(self).into_object();
        schema.metadata().title.get_or_insert_with(T::schema_name);
        let mut root = RootSchema {
            meta_schema: self.settings.meta_schema.clone(),
            definitions: self.definitions.clone(),
            schema,
        };

        for visitor in &self.settings.visitors.0 {
            visitor.visit_root_schema(&mut root)
        }

        root
    }

    /// Consumes `self` and generates a root JSON Schema for the type `T`.
    ///
    /// If `T`'s schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// include them in the returned `SchemaObject`'s [`definitions`](../schema/struct.Metadata.html#structfield.definitions)
    pub fn into_root_schema_for<T: ?Sized + JsonSchema>(mut self) -> RootSchema {
        let mut schema = T::json_schema(&mut self).into_object();
        schema.metadata().title.get_or_insert_with(T::schema_name);
        let mut root = RootSchema {
            meta_schema: self.settings.meta_schema,
            definitions: self.definitions,
            schema,
        };

        for visitor in &self.settings.visitors.0 {
            visitor.visit_root_schema(&mut root)
        }

        root
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
    /// assert_eq!(dereferenced, gen.definitions().get("MyStruct"));
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

    /// This function is only public for use by schemars_derive.
    ///
    /// It should not be considered part of the public API.
    #[doc(hidden)]
    pub fn apply_metadata(&self, schema: Schema, metadata: Option<Metadata>) -> Schema {
        match metadata {
            None => return schema,
            Some(ref metadata) if *metadata == Metadata::default() => return schema,
            Some(metadata) => {
                let mut schema_obj = schema.into_object();
                schema_obj.metadata = Some(Box::new(metadata)).merge(schema_obj.metadata);
                Schema::Object(schema_obj)
            }
        }
    }
}

/// TODO document
#[derive(Debug)]
pub struct ReplaceBoolSchemas {
    pub skip_additional_properties: bool,
}

impl Visitor for ReplaceBoolSchemas {
    fn visit_schema(&self, schema: &mut Schema) {
        if let Schema::Bool(b) = *schema {
            *schema = Schema::Bool(b).into_object().into()
        }

        visit_schema(self, schema)
    }

    fn visit_schema_object(&self, schema: &mut SchemaObject) {
        if self.skip_additional_properties {
            let mut additional_properties = None;
            if let Some(obj) = &mut schema.object {
                if let Some(ap) = &obj.additional_properties {
                    if let Schema::Bool(_) = ap.as_ref() {
                        additional_properties = obj.additional_properties.take();
                    }
                }
            }

            visit_schema_object(self, schema);

            if additional_properties.is_some() {
                schema.object().additional_properties = additional_properties;
            }
        } else {
            visit_schema_object(self, schema);
        }
    }
}

/// TODO document
#[derive(Debug)]
pub struct RemoveRefSiblings;

impl Visitor for RemoveRefSiblings {
    fn visit_schema_object(&self, schema: &mut SchemaObject) {
        visit_schema_object(self, schema);

        if let Some(reference) = schema.reference.take() {
            if schema == &SchemaObject::default() {
                schema.reference = Some(reference);
            } else {
                let ref_schema = Schema::new_ref(reference);
                let all_of = &mut schema.subschemas().all_of;
                match all_of {
                    Some(vec) => vec.push(ref_schema),
                    None => *all_of = Some(vec![ref_schema]),
                }
            }
        }
    }
}
