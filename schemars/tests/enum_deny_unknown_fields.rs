mod util;
use std::collections::BTreeMap;

use schemars::JsonSchema;
use util::*;

// Ensure that schemars_derive uses the full path to std::string::String
pub struct String;

#[derive(JsonSchema)]
struct UnitStruct;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct Struct {
    foo: i32,
    bar: bool,
}

// Outer container should always have additionalProperties: false
// `Struct` variant should have additionalProperties: false
#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(rename_all = "camelCase", deny_unknown_fields)]
enum External {
    UnitOne,
    StringMap(BTreeMap<&'static str, &'static str>),
    UnitStructNewType(UnitStruct),
    StructNewType(Struct),
    Struct {
        foo: i32,
        bar: bool,
    },
    UnitTwo,
    Tuple(i32, bool),
    // FIXME this should probably only replace the "payload" of the enum
    #[schemars(with = "i32")]
    WithInt,
}

#[test]
fn enum_external_tag() -> TestResult {
    test_default_generated_schema::<External>("enum-external-duf")
}

// Only `Struct` variant should have additionalProperties: false
#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(tag = "typeProperty", deny_unknown_fields)]
enum Internal {
    UnitOne,
    StringMap(BTreeMap<&'static str, &'static str>),
    UnitStructNewType(UnitStruct),
    StructNewType(Struct),
    Struct {
        foo: i32,
        bar: bool,
    },
    UnitTwo,
    // FIXME this should only replace the "payload" of the enum (which doesn't even make sense for unit enums!)
    #[schemars(with = "i32")]
    WithInt,
}

#[test]
fn enum_internal_tag() -> TestResult {
    test_default_generated_schema::<Internal>("enum-internal-duf")
}

// Only `Struct` variant should have additionalProperties: false
#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(untagged, deny_unknown_fields)]
enum Untagged {
    UnitOne,
    StringMap(BTreeMap<&'static str, &'static str>),
    UnitStructNewType(UnitStruct),
    StructNewType(Struct),
    Struct {
        foo: i32,
        bar: bool,
    },
    Tuple(i32, bool),
    // FIXME this should probably only replace the "payload" of the enum
    #[schemars(with = "i32")]
    WithInt,
}

#[test]
fn enum_untagged() -> TestResult {
    test_default_generated_schema::<Untagged>("enum-untagged-duf")
}

// Outer container and `Struct` variant should have additionalProperties: false
#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(tag = "t", content = "c", deny_unknown_fields)]
enum Adjacent {
    UnitOne,
    StringMap(BTreeMap<&'static str, &'static str>),
    UnitStructNewType(UnitStruct),
    StructNewType(Struct),
    Struct {
        foo: i32,
        bar: bool,
    },
    Tuple(i32, bool),
    UnitTwo,
    // FIXME this should probably only replace the "payload" of the enum
    #[schemars(with = "i32")]
    WithInt,
}

#[test]
fn enum_adjacent_tagged() -> TestResult {
    test_default_generated_schema::<Adjacent>("enum-adjacent-tagged-duf")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(tag = "typeProperty", deny_unknown_fields)]
enum SimpleInternal {
    A,
    B,
    C,
}

#[test]
fn enum_simple_internal_tag() -> TestResult {
    test_default_generated_schema::<SimpleInternal>("enum-simple-internal-duf")
}
