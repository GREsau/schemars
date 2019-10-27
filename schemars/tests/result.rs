mod util;
use schemars::JsonSchema;
use util::*;

#[derive(Debug, JsonSchema)]
struct MyStruct {
    foo: i32,
}

#[test]
fn result() -> TestResult {
    test_default_generated_schema::<Result<MyStruct, Vec<String>>>("result")
}
