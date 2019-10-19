mod util;
use schemars::{gen::SchemaGenerator, JsonSchema};
use std::ptr;

#[test]
fn dereference_i32() -> util::TestResult {
    let mut gen = SchemaGenerator::default();
    let i32_schema = gen.subschema_for::<i32>();

    let dereferenced_once = gen.dereference_once(&i32_schema)?;
    assert!(ptr::eq(dereferenced_once, &i32_schema));

    let dereferenced = gen.dereference(&i32_schema)?;
    assert!(ptr::eq(dereferenced, &i32_schema));
    Ok(())
}

#[derive(Debug, JsonSchema)]
pub struct Struct {
    foo: i32,
    bar: bool,
}

#[test]
fn dereference_struct() -> util::TestResult {
    let mut gen = SchemaGenerator::default();
    let struct_ref_schema = gen.subschema_for::<Struct>();
    let struct_schema = gen.definitions().get(&<Struct>::schema_name()).unwrap();

    let dereferenced_once = gen.dereference_once(&struct_ref_schema)?;
    assert!(ptr::eq(dereferenced_once, struct_schema));

    let dereferenced = gen.dereference(&struct_ref_schema)?;
    assert!(ptr::eq(dereferenced, struct_schema));
    Ok(())
}
