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

#[derive(Debug, JsonSchema)]
#[schemars(rename_all = "camelCase")]
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
    test_default_generated_schema::<External>("enum-external")
}

#[derive(Debug, JsonSchema)]
#[schemars(tag = "typeProperty")]
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
    test_default_generated_schema::<Internal>("enum-internal")
}

#[derive(Debug, JsonSchema)]
#[schemars(untagged)]
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
    test_default_generated_schema::<Untagged>("enum-untagged")
}

#[derive(Debug, JsonSchema)]
#[schemars(tag = "t", content = "c")]
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
    test_default_generated_schema::<Adjacent>("enum-adjacent-tagged")
}

#[derive(Debug, JsonSchema)]
#[schemars(tag = "typeProperty")]
pub enum SimpleInternal {
    A,
    B,
    C,
}

#[test]
fn enum_simple_internal_tag() -> TestResult {
    test_default_generated_schema::<SimpleInternal>("enum-simple-internal")
}

#[derive(Debug, JsonSchema)]
pub enum Aliased {
    #[schemars(alias = "v1", alias = "variant1")]
    V1,
    #[schemars(alias = "v2", alias = "variant2")]
    V2,
}
#[test]
fn enum_aliased() -> TestResult {
    test_default_generated_schema::<Aliased>("enum-aliased")
}
