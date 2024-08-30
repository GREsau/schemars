mod util;
use schemars::{generate::SchemaSettings, JsonSchema};
use util::*;

#[allow(dead_code)]
#[derive(Default, JsonSchema)]
struct MyStruct {
    // TODO
    #[schemars(skip_deserializing)]
    read_only: bool,
    // TODO
    #[schemars(skip_serializing)]
    write_only: bool,
    // TODO add to required for serialize but not deserialize
    #[schemars(default)]
    default: bool,
    // TODO add to required for deserialize but not serialize
    #[schemars(skip_serializing_if = "anything")]
    skip_serializing_if: bool,
    #[schemars(rename(serialize = "ser_renamed", deserialize = "de_renamed"))]
    renamed: bool,
    // TODO
    #[schemars(deserialize_with = "i8")]
    deserialize_with_i8: bool,
    // TODO
    #[schemars(serialize_with = "u8")]
    serialize_with_u8: bool,
}

#[test]
fn contract_deserialize() -> TestResult {
    test_generated_schema::<MyStruct>(
        "contract_deserialize",
        SchemaSettings::default().for_deserialize(),
    )
}

#[test]
fn contract_serialize() -> TestResult {
    test_generated_schema::<MyStruct>(
        "contract_serialize",
        SchemaSettings::default().for_serialize(),
    )
}
