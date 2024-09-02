mod util;
use schemars::JsonSchema;
use serde_json::Value;
use std::collections::BTreeMap;
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct Flat {
    f: f32,
    b: bool,
    s: String,
    #[serde(default)]
    os: String,
    v: Vec<i32>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(rename = "Flat")]
struct Deep1 {
    f: f32,
    #[schemars(flatten)]
    deep2: Deep2,
    v: Vec<i32>,
}

#[allow(clippy::option_option, dead_code)]
#[derive(JsonSchema)]
struct Deep2 {
    b: bool,
    #[serde(flatten)]
    deep3: Deep3,
    #[serde(flatten)]
    deep4: Box<Option<Option<Box<Deep4>>>>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct Deep3 {
    s: &'static str,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct Deep4 {
    #[serde(default)]
    os: &'static str,
}

#[test]
fn test_flat_schema() -> TestResult {
    test_default_generated_schema::<Flat>("flatten")
}

#[test]
fn test_flattened_schema() -> TestResult {
    // intentionally using the same file as test_flat_schema, as the schema should be identical
    test_default_generated_schema::<Deep1>("flatten")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct FlattenValue {
    flag: bool,
    #[serde(flatten)]
    value: Value,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(rename = "FlattenValue")]
struct FlattenMap {
    flag: bool,
    #[serde(flatten)]
    value: BTreeMap<String, Value>,
}

#[test]
fn test_flattened_value() -> TestResult {
    test_default_generated_schema::<FlattenValue>("flattened_value")
}

#[test]
fn test_flattened_map() -> TestResult {
    // intentionally using the same file as test_flattened_value, as the schema should be identical
    test_default_generated_schema::<FlattenMap>("flattened_value")
}

#[derive(JsonSchema)]
pub struct OuterAllowUnknownFields {
    pub outer_field: bool,
    #[serde(flatten)]
    pub middle: MiddleDenyUnknownFields,
}

#[derive(JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MiddleDenyUnknownFields {
    pub middle_field: bool,
    #[serde(flatten)]
    pub inner: InnerAllowUnknownFields,
}

#[derive(JsonSchema)]
pub struct InnerAllowUnknownFields {
    pub inner_field: bool,
}

#[test]
fn test_flattened_struct_deny_unknown_fields() -> TestResult {
    test_default_generated_schema::<(OuterAllowUnknownFields, MiddleDenyUnknownFields)>(
        "test_flattened_struct_deny_unknown_fields",
    )
}
