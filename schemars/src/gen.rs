/*!
JSON Schema generator and settings.

This module is useful if you want more control over how the schema generated than the [`schema_for!`] macro gives you.
There are two main types in this module:
* [`SchemaSettings`], which defines what JSON Schema features should be used when generating schemas (for example, how `Option`s should be represented).
* [`SchemaGenerator`], which manages the generation of a schema document.
*/

use crate::schema::*;
use crate::{visit::*, JsonSchema};
use dyn_clone::DynClone;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::{any::Any, collections::HashSet, fmt::Debug};

/// Settings to customize how Schemas are generated.
///
/// The default settings currently conform to [JSON Schema Draft 7](https://json-schema.org/specification-links.html#draft-7), but this is liable to change in a future version of Schemars if support for other JSON Schema versions is added.
/// If you require your generated schemas to conform to draft 7, consider using the [`draft07`](#method.draft07) method.
#[derive(Debug, Clone)]
#[non_exhaustive]
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
    /// A list of visitors that get applied to all generated root schemas.
    pub visitors: Vec<Box<dyn GenVisitor>>,
    /// Inline all subschemas instead of using references.
    ///
    /// Some references may still be generated in schemas for recursive types.
    ///
    /// Defaults to `false`.
    pub inline_subschemas: bool,
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
            visitors: vec![Box::new(RemoveRefSiblings)],
            inline_subschemas: false,
        }
    }

    /// Creates `SchemaSettings` that conform to [JSON Schema 2019-09](https://json-schema.org/specification-links.html#2019-09-formerly-known-as-draft-8).
    pub fn draft2019_09() -> SchemaSettings {
        SchemaSettings {
            option_nullable: false,
            option_add_null_type: true,
            definitions_path: "#/definitions/".to_owned(),
            meta_schema: Some("https://json-schema.org/draft/2019-09/schema".to_owned()),
            visitors: Vec::new(),
            inline_subschemas: false,
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
            visitors: vec![
                Box::new(RemoveRefSiblings),
                Box::new(ReplaceBoolSchemas {
                    skip_additional_properties: true,
                }),
                Box::new(SetSingleExample),
            ],
            inline_subschemas: false,
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

    /// Appends the given visitor to the list of [visitors](SchemaSettings::visitors) for these `SchemaSettings`.
    pub fn with_visitor(mut self, visitor: impl Visitor + Debug + Clone + 'static) -> Self {
        self.visitors.push(Box::new(visitor));
        self
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
#[derive(Debug, Default)]
pub struct SchemaGenerator {
    settings: SchemaSettings,
    definitions: BTreeMap<String, Schema>,
    pending_schema_ids: HashSet<Cow<'static, str>>,
    schema_id_to_name: HashMap<Cow<'static, str>, String>,
    used_schema_names: HashSet<String>,
}

impl Clone for SchemaGenerator {
    fn clone(&self) -> Self {
        Self {
            settings: self.settings.clone(),
            definitions: self.definitions.clone(),
            pending_schema_ids: HashSet::new(),
            schema_id_to_name: HashMap::new(),
            used_schema_names: HashSet::new(),
        }
    }
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

    /// Generates a JSON Schema for the type `T`, and returns either the schema itself or a `$ref` schema referencing `T`'s schema.
    ///
    /// If `T` is [referenceable](JsonSchema::is_referenceable), this will add `T`'s schema to this generator's definitions, and
    /// return a `$ref` schema referencing that schema. Otherwise, this method behaves identically to [`JsonSchema::json_schema`].
    ///
    /// If `T`'s schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// add them to the `SchemaGenerator`'s schema definitions.
    pub fn subschema_for<T: ?Sized + JsonSchema>(&mut self) -> Schema {
        let id = T::schema_id();
        let return_ref = T::is_referenceable()
            && (!self.settings.inline_subschemas || self.pending_schema_ids.contains(&id));

        if return_ref {
            let name = match self.schema_id_to_name.get(&id).cloned() {
                Some(n) => n,
                None => {
                    let base_name = T::schema_name();
                    let mut name = String::new();

                    if self.used_schema_names.contains(&base_name) {
                        for i in 2.. {
                            name = format!("{}{}", base_name, i);
                            if !self.used_schema_names.contains(&name) {
                                break;
                            }
                        }
                    } else {
                        name = base_name;
                    }

                    self.used_schema_names.insert(name.clone());
                    self.schema_id_to_name.insert(id.clone(), name.clone());
                    name
                }
            };

            let reference = format!("{}{}", self.settings.definitions_path, name);
            if !self.definitions.contains_key(&name) {
                self.insert_new_subschema_for::<T>(name, id);
            }
            Schema::new_ref(reference)
        } else {
            self.json_schema_internal::<T>(id)
        }
    }

    fn insert_new_subschema_for<T: ?Sized + JsonSchema>(
        &mut self,
        name: String,
        id: Cow<'static, str>,
    ) {
        let dummy = false.into();
        // insert into definitions BEFORE calling json_schema to avoid infinite recursion
        self.definitions.insert(name.clone(), dummy);

        let schema = self.json_schema_internal::<T>(id);

        self.definitions.insert(name, schema);
    }

    /// Borrows the collection of all [referenceable](JsonSchema::is_referenceable) schemas that have been generated.
    ///
    /// The keys of the returned `BTreeMap` are the [schema names](JsonSchema::schema_name), and the values are the schemas
    /// themselves.
    pub fn definitions(&self) -> &BTreeMap<String, Schema> {
        &self.definitions
    }

    /// Mutably borrows the collection of all [referenceable](JsonSchema::is_referenceable) schemas that have been generated.
    ///
    /// The keys of the returned `BTreeMap` are the [schema names](JsonSchema::schema_name), and the values are the schemas
    /// themselves.
    pub fn definitions_mut(&mut self) -> &mut BTreeMap<String, Schema> {
        &mut self.definitions
    }

    /// Returns the collection of all [referenceable](JsonSchema::is_referenceable) schemas that have been generated,
    /// leaving an empty map in its place.
    ///
    /// The keys of the returned `BTreeMap` are the [schema names](JsonSchema::schema_name), and the values are the schemas
    /// themselves.
    pub fn take_definitions(&mut self) -> BTreeMap<String, Schema> {
        std::mem::take(&mut self.definitions)
    }

    /// Returns an iterator over the [visitors](SchemaSettings::visitors) being used by this `SchemaGenerator`.
    pub fn visitors_mut(&mut self) -> impl Iterator<Item = &mut dyn GenVisitor> {
        self.settings.visitors.iter_mut().map(|v| v.as_mut())
    }

    /// Generates a root JSON Schema for the type `T`.
    ///
    /// If `T`'s schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// add them to the `SchemaGenerator`'s schema definitions and include them in the returned `SchemaObject`'s
    /// [`definitions`](../schema/struct.Metadata.html#structfield.definitions)
    pub fn root_schema_for<T: ?Sized + JsonSchema>(&mut self) -> Schema {
        let mut schema = self.json_schema_internal::<T>(T::schema_id());

        let object = schema.ensure_object();

        object
            .entry("title")
            .or_insert_with(|| T::schema_name().into());

        if let Some(meta_schema) = self.settings.meta_schema.as_deref() {
            object.insert("$schema".into(), meta_schema.into());
        }

        if !self.definitions.is_empty() {
            object.insert(
                "definitions".into(),
                serde_json::Value::Object(
                    self.definitions
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone().into()))
                        .collect(),
                ),
            );
        }

        for visitor in &mut self.settings.visitors {
            visitor.visit_schema(&mut schema);
        }

        schema
    }

    /// Consumes `self` and generates a root JSON Schema for the type `T`.
    ///
    /// If `T`'s schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// include them in the returned `SchemaObject`'s [`definitions`](../schema/struct.Metadata.html#structfield.definitions)
    pub fn into_root_schema_for<T: ?Sized + JsonSchema>(mut self) -> Schema {
        let mut schema = self.json_schema_internal::<T>(T::schema_id());

        let object = schema.ensure_object();

        object
            .entry("title")
            .or_insert_with(|| T::schema_name().into());

        if let Some(meta_schema) = self.settings.meta_schema {
            object.insert("$schema".into(), meta_schema.into());
        }

        if !self.definitions.is_empty() {
            object.insert(
                "definitions".into(),
                serde_json::Value::Object(
                    self.definitions
                        .into_iter()
                        .map(|(k, v)| (k, v.into()))
                        .collect(),
                ),
            );
        }

        for visitor in &mut self.settings.visitors {
            visitor.visit_schema(&mut schema);
        }

        schema
    }

    /// Generates a root JSON Schema for the given example value.
    ///
    /// If the value implements [`JsonSchema`](crate::JsonSchema), then prefer using the [`root_schema_for()`](Self::root_schema_for())
    /// function which will generally produce a more precise schema, particularly when the value contains any enums.
    pub fn root_schema_for_value<T: ?Sized + Serialize>(
        &mut self,
        value: &T,
    ) -> Result<Schema, serde_json::Error> {
        let mut schema = value.serialize(crate::ser::Serializer {
            gen: self,
            include_title: true,
        })?;

        let object = schema.ensure_object();

        if let Ok(example) = serde_json::to_value(value) {
            object.insert("examples".into(), vec![example].into());
        }

        if let Some(meta_schema) = self.settings.meta_schema.as_deref() {
            object.insert("$schema".into(), meta_schema.into());
        }

        if !self.definitions.is_empty() {
            object.insert(
                "definitions".into(),
                serde_json::Value::Object(
                    self.definitions
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone().into()))
                        .collect(),
                ),
            );
        }

        for visitor in &mut self.settings.visitors {
            visitor.visit_schema(&mut schema);
        }

        Ok(schema)
    }

    /// Consumes `self` and generates a root JSON Schema for the given example value.
    ///
    /// If the value  implements [`JsonSchema`](crate::JsonSchema), then prefer using the [`into_root_schema_for()!`](Self::into_root_schema_for())
    /// function which will generally produce a more precise schema, particularly when the value contains any enums.
    pub fn into_root_schema_for_value<T: ?Sized + Serialize>(
        mut self,
        value: &T,
    ) -> Result<Schema, serde_json::Error> {
        let mut schema = value.serialize(crate::ser::Serializer {
            gen: &mut self,
            include_title: true,
        })?;

        let object = schema.ensure_object();

        if let Ok(example) = serde_json::to_value(value) {
            object.insert("examples".into(), vec![example].into());
        }

        if let Some(meta_schema) = self.settings.meta_schema {
            object.insert("$schema".into(), meta_schema.into());
        }

        if !self.definitions.is_empty() {
            object.insert(
                "definitions".into(),
                serde_json::Value::Object(
                    self.definitions
                        .into_iter()
                        .map(|(k, v)| (k, v.into()))
                        .collect(),
                ),
            );
        }

        for visitor in &mut self.settings.visitors {
            visitor.visit_schema(&mut schema);
        }

        Ok(schema)
    }

    fn json_schema_internal<T: ?Sized + JsonSchema>(&mut self, id: Cow<'static, str>) -> Schema {
        struct PendingSchemaState<'a> {
            gen: &'a mut SchemaGenerator,
            id: Cow<'static, str>,
            did_add: bool,
        }

        impl<'a> PendingSchemaState<'a> {
            fn new(gen: &'a mut SchemaGenerator, id: Cow<'static, str>) -> Self {
                let did_add = gen.pending_schema_ids.insert(id.clone());
                Self { gen, id, did_add }
            }
        }

        impl Drop for PendingSchemaState<'_> {
            fn drop(&mut self) {
                if self.did_add {
                    self.gen.pending_schema_ids.remove(&self.id);
                }
            }
        }

        let pss = PendingSchemaState::new(self, id);
        T::json_schema(pss.gen)
    }
}

/// A [Visitor](Visitor) which implements additional traits required to be included in a [SchemaSettings].
///
/// You will rarely need to use this trait directly as it is automatically implemented for any type which implements all of:
/// - [`Visitor`]
/// - [`std::fmt::Debug`]
/// - [`std::any::Any`] (implemented for all `'static` types)
/// - [`std::clone::Clone`]
///
/// # Example
/// ```
/// use schemars::visit::Visitor;
/// use schemars::gen::GenVisitor;
///
/// #[derive(Debug, Clone)]
/// struct MyVisitor;
///
/// impl Visitor for MyVisitor { }
///
/// let v: &dyn GenVisitor = &MyVisitor;
/// assert!(v.as_any().is::<MyVisitor>());
/// ```
pub trait GenVisitor: Visitor + Debug + DynClone + Any {
    /// Upcasts this visitor into an `Any`, which can be used to inspect and manipulate it as its concrete type.
    fn as_any(&self) -> &dyn Any;
}

dyn_clone::clone_trait_object!(GenVisitor);

impl<T> GenVisitor for T
where
    T: Visitor + Debug + Clone + Any,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}
