mod util;
use schemars::gen::SchemaSettings;
use schemars::schema::Schema;
use util::*;

#[test]
fn schema_matches_default_settings() -> TestResult {
    test_default_generated_schema::<Schema>("schema")
}

#[test]
fn schema_matches_openapi3() -> TestResult {
    test_generated_schema::<Schema>("schema-openapi3", SchemaSettings::openapi3())
}
