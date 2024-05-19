mod util;
use schemars::gen::SchemaSettings;
use schemars::JsonSchema;
use serde_json::Value;
use std::collections::BTreeMap;
use util::*;

#[derive(JsonSchema)]
pub struct Outer {
    #[schemars(example = "eight", example = "null")]
    pub int: i32,
    pub values: BTreeMap<&'static str, Value>,
    pub value: Value,
    pub inner: Option<Inner>,
    pub tuples: Vec<(u8, i64)>,
}

#[derive(JsonSchema)]
pub enum Inner {
    UndocumentedUnit1,
    UndocumentedUnit2,
    /// This is a documented unit variant
    DocumentedUnit,
    ValueNewType(Value),
}

fn eight() -> i32 {
    8
}

fn null() {}

#[test]
fn schema_matches_draft07() -> TestResult {
    test_generated_schema::<Outer>("schema_settings", SchemaSettings::draft07())
}

#[test]
fn schema_matches_2019_09() -> TestResult {
    test_generated_schema::<Outer>("schema_settings-2019_09", SchemaSettings::draft2019_09())
}

#[test]
fn schema_matches_2020_12() -> TestResult {
    test_generated_schema::<Outer>("schema_settings-2020_12", SchemaSettings::draft2020_12())
}

#[test]
fn schema_matches_openapi3() -> TestResult {
    test_generated_schema::<Outer>("schema_settings-openapi3", SchemaSettings::openapi3())
}
