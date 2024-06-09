mod util;
use enumset1::{EnumSet, EnumSetType};
use schemars::JsonSchema;
use util::*;

// needed to derive EnumSetType when using a crate alias
extern crate enumset1 as enumset;

#[derive(EnumSetType, JsonSchema)]
enum Foo {
    Bar,
    Baz,
}

#[test]
fn enumset() -> TestResult {
    test_default_generated_schema::<EnumSet<Foo>>("enumset")
}
