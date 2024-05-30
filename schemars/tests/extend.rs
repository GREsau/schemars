mod util;
use schemars::JsonSchema;
use serde_json::Value;
use util::*;

const THREE: f64 = 3.0;

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(extend("msg" = concat!("hello ", "world"), "obj" = {"array": [null, ()]}))]
#[schemars(extend("3" = THREE), extend("pi" = THREE + 0.14))]
struct Struct {
    #[schemars(extend("foo" = "bar"))]
    value: Value,
    #[schemars(extend("type" = "overridden"))]
    int: i32,
}

#[test]
fn doc_comments_struct() -> TestResult {
    test_default_generated_schema::<Struct>("extend_struct")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(extend("foo" = "bar"))]
enum External {
    #[schemars(extend("foo" = "bar"))]
    Unit,
    #[schemars(extend("foo" = "bar"))]
    NewType(Value),
    #[schemars(extend("foo" = "bar"))]
    Tuple(i32, bool),
    #[schemars(extend("foo" = "bar"))]
    Struct { i: i32, b: bool },
}

#[test]
fn doc_comments_enum_external() -> TestResult {
    test_default_generated_schema::<External>("extend_enum_external")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(tag = "typeProperty", extend("foo" = "bar"))]
enum Internal {
    #[schemars(extend("foo" = "bar"))]
    Unit,
    #[schemars(extend("foo" = "bar"))]
    NewType(Value),
    #[schemars(extend("foo" = "bar"))]
    Struct { i: i32, b: bool },
}

#[test]
fn doc_comments_enum_internal() -> TestResult {
    test_default_generated_schema::<Internal>("extend_enum_internal")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(untagged, extend("foo" = "bar"))]
enum Untagged {
    #[schemars(extend("foo" = "bar"))]
    Unit,
    #[schemars(extend("foo" = "bar"))]
    NewType(Value),
    #[schemars(extend("foo" = "bar"))]
    Tuple(i32, bool),
    #[schemars(extend("foo" = "bar"))]
    Struct { i: i32, b: bool },
}

#[test]
fn doc_comments_enum_untagged() -> TestResult {
    test_default_generated_schema::<Untagged>("extend_enum_untagged")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(tag = "t", content = "c", extend("foo" = "bar"))]
enum Adjacent {
    #[schemars(extend("foo" = "bar"))]
    Unit,
    #[schemars(extend("foo" = "bar"))]
    NewType(Value),
    #[schemars(extend("foo" = "bar"))]
    Tuple(i32, bool),
    #[schemars(extend("foo" = "bar"))]
    Struct { i: i32, b: bool },
}

#[test]
fn doc_comments_enum_adjacent() -> TestResult {
    test_default_generated_schema::<Adjacent>("extend_enum_adjacent")
}
