mod util;
use schemars::JsonSchema;
use util::*;

#[derive(Debug, JsonSchema)]
pub struct OuterStruct {
    inner: TransparentStruct,
}

#[derive(Debug, JsonSchema)]
#[serde(transparent)]
pub struct TransparentStruct {
    #[serde(with = "TransparentNewType")]
    inner: (),
}

#[derive(Debug, JsonSchema)]
#[schemars(transparent)]
pub struct TransparentNewType(Option<InnerStruct>);

#[derive(Debug, JsonSchema)]
pub struct InnerStruct(String, i32);

#[test]
fn transparent_struct() -> TestResult {
    test_default_generated_schema::<OuterStruct>("transparent-struct")
}
