use crate::prelude::*;
use pretty_assertions::assert_eq;
use schemars::{generate::SchemaSettings, Schema, SchemaGenerator};

mod a {
    use super::*;

    #[derive(JsonSchema, Serialize)]
    pub struct Config {
        pub test: String,
    }
}

mod b {
    use super::*;

    #[derive(JsonSchema, Serialize)]
    pub struct Config {
        pub test2: String,
    }
}

#[derive(JsonSchema, Serialize)]
struct MyStruct {
    foo: i32,
}

fn manual_definition() -> Value {
    json!({
        "description": "inserted manually"
    })
}

#[test]
fn manual_definition_is_not_shadowed() {
    let mut generator = SchemaGenerator::default();
    generator
        .definitions_mut()
        .insert("MyStruct".to_owned(), manual_definition());

    let subschema = generator.subschema_for::<MyStruct>();

    assert_eq!(subschema, Schema::new_ref("#/$defs/MyStruct2".to_owned()));
    assert_eq!(
        generator.definitions().get("MyStruct"),
        Some(&manual_definition())
    );
    assert_eq!(
        generator.definitions().get("MyStruct2"),
        Some(&MyStruct::json_schema(&mut SchemaGenerator::default()).to_value())
    );
}

#[test]
fn manual_definition_is_not_overwritten() {
    let mut generator = SchemaGenerator::default();
    generator
        .definitions_mut()
        .insert("Config".to_owned(), manual_definition());

    generator.subschema_for::<a::Config>();
    generator.subschema_for::<b::Config>();

    assert_eq!(
        generator.definitions().get("Config"),
        Some(&manual_definition())
    );
    assert!(generator.definitions().contains_key("Config2"));
    assert!(generator.definitions().contains_key("Config3"));
}

#[test]
fn definition_name_for() {
    let mut generator = SchemaGenerator::default();

    assert_eq!(
        generator.definition_name_for::<a::Config>().as_deref(),
        Some("Config")
    );
    assert_eq!(
        generator.definition_name_for::<b::Config>().as_deref(),
        Some("Config2")
    );
    // Asking again returns the same name
    assert_eq!(
        generator.definition_name_for::<a::Config>().as_deref(),
        Some("Config")
    );

    // `definition_name_for` reserves the name, but does not add the definition
    assert!(generator.definitions().is_empty());

    assert_eq!(
        generator.subschema_for::<b::Config>(),
        Schema::new_ref("#/$defs/Config2".to_owned())
    );
    assert_eq!(
        generator.subschema_for::<a::Config>(),
        Schema::new_ref("#/$defs/Config".to_owned())
    );
    assert_eq!(generator.definitions().len(), 2);
    assert_eq!(
        generator.definitions().get("Config"),
        Some(&a::Config::json_schema(&mut SchemaGenerator::default()).to_value())
    );
    assert_eq!(
        generator.definitions().get("Config2"),
        Some(&b::Config::json_schema(&mut SchemaGenerator::default()).to_value())
    );
}

#[test]
fn definition_name_for_after_subschema_for() {
    let mut generator = SchemaGenerator::default();

    generator.subschema_for::<a::Config>();
    generator.subschema_for::<b::Config>();

    assert_eq!(
        generator.definition_name_for::<a::Config>().as_deref(),
        Some("Config")
    );
    assert_eq!(
        generator.definition_name_for::<b::Config>().as_deref(),
        Some("Config2")
    );
}

#[test]
fn definition_name_for_avoids_manual_definition() {
    let mut generator = SchemaGenerator::default();
    generator
        .definitions_mut()
        .insert("MyStruct".to_owned(), manual_definition());

    assert_eq!(
        generator.definition_name_for::<MyStruct>().as_deref(),
        Some("MyStruct2")
    );
}

#[test]
fn definition_name_for_inlined() {
    let mut generator = SchemaGenerator::default();
    assert_eq!(generator.definition_name_for::<i32>(), None);
    assert_eq!(generator.definition_name_for::<String>(), None);

    let mut generator = SchemaSettings::default()
        .with(|s| s.inline_subschemas = true)
        .into_generator();
    assert_eq!(generator.definition_name_for::<MyStruct>(), None);
}

#[test]
fn replace_definition() {
    let mut generator = SchemaGenerator::default();
    generator.subschema_for::<a::Config>();
    generator.subschema_for::<b::Config>();

    let name = generator.definition_name_for::<b::Config>().unwrap();
    generator
        .definitions_mut()
        .insert(name.into_owned(), manual_definition());

    assert_eq!(
        generator.subschema_for::<b::Config>(),
        Schema::new_ref("#/$defs/Config2".to_owned())
    );
    assert_eq!(
        generator.definitions().get("Config2"),
        Some(&manual_definition())
    );
    assert_ne!(
        generator.definitions().get("Config"),
        Some(&manual_definition())
    );
}

#[test]
fn cloned_generator_reuses_definitions() {
    let mut generator = SchemaGenerator::default();
    generator.subschema_for::<a::Config>();
    generator.subschema_for::<b::Config>();

    let mut cloned = generator.clone();
    assert_eq!(
        cloned.subschema_for::<b::Config>(),
        Schema::new_ref("#/$defs/Config2".to_owned())
    );
    assert_eq!(
        cloned.subschema_for::<a::Config>(),
        Schema::new_ref("#/$defs/Config".to_owned())
    );
    assert_eq!(cloned.definitions(), generator.definitions());
}
