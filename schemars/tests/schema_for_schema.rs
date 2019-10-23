mod util;
use schemars::gen::SchemaSettings;
use schemars::schema::RootSchema;
use util::*;

#[test]
fn schema_matches_default_settings() -> TestResult {
    test_default_generated_schema::<RootSchema>("schema")
}

#[test]
fn schema_matches_openapi3() -> TestResult {
    test_generated_schema::<RootSchema>("schema-openapi3", SchemaSettings::openapi3())
}
