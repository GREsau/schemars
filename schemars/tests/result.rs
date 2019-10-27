mod util;
use schemars::JsonSchema;
use util::*;

#[derive(Debug, JsonSchema)]
struct MyStruct {
    foo: i32,
}

#[derive(Debug, JsonSchema)]
struct Container {
    result1: Result<MyStruct, Vec<String>>,
    result2: Result<bool, ()>,
}

#[test]
fn result() -> TestResult {
    test_default_generated_schema::<Container>("result")
}
