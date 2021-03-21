mod util;
use schemars::JsonSchema;
use util::*;

fn schema_fn(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    <bool>::json_schema(gen)
}

#[derive(Debug)]
struct DoesntImplementJsonSchema;

#[derive(Debug, JsonSchema)]
pub struct Struct {
    #[schemars(schema_with = "schema_fn")]
    foo: DoesntImplementJsonSchema,
    bar: i32,
    #[schemars(schema_with = "schema_fn")]
    baz: DoesntImplementJsonSchema,
}

#[test]
fn struct_normal() -> TestResult {
    test_default_generated_schema::<Struct>("schema_with-struct")
}

#[derive(Debug, JsonSchema)]
pub struct Tuple(
    #[schemars(schema_with = "schema_fn")] DoesntImplementJsonSchema,
    i32,
    #[schemars(schema_with = "schema_fn")] DoesntImplementJsonSchema,
);

#[test]
fn struct_tuple() -> TestResult {
    test_default_generated_schema::<Tuple>("schema_with-tuple")
}

#[derive(Debug, JsonSchema)]
pub struct Newtype(#[schemars(schema_with = "schema_fn")] DoesntImplementJsonSchema);

#[test]
fn struct_newtype() -> TestResult {
    test_default_generated_schema::<Newtype>("schema_with-newtype")
}

#[derive(Debug, JsonSchema)]
#[schemars(transparent)]
pub struct TransparentNewtype(#[schemars(schema_with = "schema_fn")] DoesntImplementJsonSchema);

#[test]
fn struct_transparent_newtype() -> TestResult {
    test_default_generated_schema::<TransparentNewtype>("schema_with-transparent-newtype")
}
