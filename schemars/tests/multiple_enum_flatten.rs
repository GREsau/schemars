mod util;
use schemars::JsonSchema;
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(rename = "Flat")]
struct Flat {
    f: f32,
    #[schemars(flatten)]
    e1: Enum1,
    #[schemars(flatten)]
    e2: Enum2,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
enum Enum1 {
    B(bool),
    S(String),
}

#[allow(dead_code)]
#[derive(JsonSchema)]
enum Enum2 {
    U(u32),
    F(f64)
}

#[test]
fn test_flat_schema() -> TestResult {
    test_default_generated_schema::<Flat>("multiple_enum_flatten")
}
