mod util;
use schemars::MakeSchema;
use util::*;

#[derive(Debug, MakeSchema)]
#[schemars(rename_all = "camelCase")]
pub enum External {
    UnitOne,
    String(String),
    Struct{ foo: i32, bar: bool },
    UnitTwo,
}

#[test]
fn enum_external_tag() -> TestResult {
    test_default_generated_schema::<External>("enum-external")
}
/*
#[derive(Debug, MakeSchema)]
#[schemars(tag = "typeProperty")]
pub enum Internal {
    UnitOne,
    String(String),
    Struct{ foo: i32, bar: bool },
    UnitTwo,
}

#[test]
fn enum_internal_tag() -> TestResult {
    test_default_generated_schema::<Internal>("enum-internal")
}
*/
#[derive(Debug, MakeSchema)]
#[schemars(untagged)]
pub enum Untagged {
    UnitOne,
    String(String),
    Struct{ foo: i32, bar: bool }
}

#[test]
fn enum_untagged() -> TestResult {
    test_default_generated_schema::<Untagged>("enum-untagged")
}
