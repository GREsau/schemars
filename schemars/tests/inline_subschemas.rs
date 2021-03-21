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
    let mut settings = SchemaSettings::default();
    settings.inline_subschemas = true;
    test_generated_schema::<MyJob>("inline-subschemas", settings)
}

#[derive(Debug, JsonSchema)]
pub struct RecursiveOuter {
    pub direct: Option<Box<RecursiveOuter>>,
    pub indirect: Option<Box<RecursiveInner>>,
}

#[derive(Debug, JsonSchema)]
pub struct RecursiveInner {
    pub recursive: RecursiveOuter,
}

#[test]
fn struct_recursive() -> TestResult {
    let mut settings = SchemaSettings::default();
    settings.inline_subschemas = true;
    test_generated_schema::<RecursiveOuter>("inline-subschemas-recursive", settings)
}
