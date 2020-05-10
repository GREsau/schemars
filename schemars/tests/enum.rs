mod util;
use schemars::{JsonSchema, Map};
use util::*;

#[derive(Debug, JsonSchema)]
pub struct UnitStruct;

#[derive(Debug, JsonSchema)]
pub struct Struct {
    foo: i32,
    bar: bool,
}

#[derive(Debug, JsonSchema)]
#[schemars(rename_all = "camelCase")]
pub enum External {
    UnitOne,
    StringMap(Map<String, String>),
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
    test_default_generated_schema::<External>("enum-external")
}

#[derive(Debug, JsonSchema)]
#[schemars(tag = "typeProperty")]
pub enum Internal {
    UnitOne,
    StringMap(Map<String, String>),
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
    test_default_generated_schema::<Internal>("enum-internal")
}

#[derive(Debug, JsonSchema)]
#[schemars(untagged)]
pub enum Untagged {
    UnitOne,
    StringMap(Map<String, String>),
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
    test_default_generated_schema::<Untagged>("enum-untagged")
}

#[derive(Debug, JsonSchema)]
#[schemars(tag = "t", content = "c")]
pub enum Adjacent {
    UnitOne,
    StringMap(Map<String, String>),
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
    test_default_generated_schema::<Adjacent>("enum_adjacent_tagged-untagged")
}
