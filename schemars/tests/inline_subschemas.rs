mod util;
use schemars::gen::SchemaSettings;
use schemars::JsonSchema;
use util::*;

#[derive(Debug, JsonSchema)]
pub struct MyJob {
    pub spec: MyJobSpec,
}

#[derive(Debug, JsonSchema)]
pub struct MyJobSpec {
    pub replicas: u32,
}

#[test]
fn struct_normal() -> TestResult {
    let mut settings = SchemaSettings::openapi3();
    settings.inline_subschemas = true;
    test_generated_schema::<MyJob>("inline-subschemas", settings)
}
