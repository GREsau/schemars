mod util;
use schemars::{JsonSchema, Map};
use util::*;

// Ensure that schemars_derive uses the full path to std::string::String
pub struct String;

#[derive(Debug, JsonSchema)]
pub struct UnitStruct;

#[derive(Debug, JsonSchema)]
pub struct Struct {
    foo: i32,
    bar: bool,
}

// Outer container should always have additionalPropreties: false
// `Struct` variant should have additionalPropreties: false
#[derive(Debug, JsonSchema)]
#[schemars(rename_all = "camelCase", deny_unknown_fields)]
pub enum External {
    UnitOne,
    StringMap(Map<&'static str, &'static str>),
    UnitStructNewType(UnitStruct),
    StructNewType(Struct),
    Struct {
        foo: i32,
        bar: bool,
    },
    UnitTwo,
    Tuple(i32, bool),
    #[schemars(with = "i32")]
    WithInt,
}

#[test]
fn enum_external_tag() -> TestResult {
    test_default_generated_schema::<External>("enum-external-duf")
}

// Only `Struct` variant should have additionalPropreties: false
#[derive(Debug, JsonSchema)]
#[schemars(tag = "typeProperty", deny_unknown_fields)]
pub enum Internal {
    UnitOne,
    StringMap(Map<&'static str, &'static str>),
    UnitStructNewType(UnitStruct),
    StructNewType(Struct),
    Struct {
        foo: i32,
        bar: bool,
    },
    UnitTwo,
    #[schemars(with = "i32")]
    WithInt,
}

#[test]
fn enum_internal_tag() -> TestResult {
    test_default_generated_schema::<Internal>("enum-internal-duf")
}

// Only `Struct` variant should have additionalPropreties: false
#[derive(Debug, JsonSchema)]
#[schemars(untagged, deny_unknown_fields)]
pub enum Untagged {
    UnitOne,
    StringMap(Map<&'static str, &'static str>),
    UnitStructNewType(UnitStruct),
    StructNewType(Struct),
    Struct {
        foo: i32,
        bar: bool,
    },
    Tuple(i32, bool),
    #[schemars(with = "i32")]
    WithInt,
}

#[test]
fn enum_untagged() -> TestResult {
    test_default_generated_schema::<Untagged>("enum-untagged-duf")
}

// Outer container and `Struct` variant should have additionalPropreties: false
#[derive(Debug, JsonSchema)]
#[schemars(tag = "t", content = "c", deny_unknown_fields)]
pub enum Adjacent {
    UnitOne,
    StringMap(Map<&'static str, &'static str>),
    UnitStructNewType(UnitStruct),
    StructNewType(Struct),
    Struct {
        foo: i32,
        bar: bool,
    },
    Tuple(i32, bool),
    UnitTwo,
    #[schemars(with = "i32")]
    WithInt,
}

#[test]
fn enum_adjacent_tagged() -> TestResult {
    test_default_generated_schema::<Adjacent>("enum-adjacent-tagged-duf")
}
