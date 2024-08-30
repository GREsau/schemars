mod util;
use schemars::{generate::SchemaSettings, JsonSchema};
use util::*;

#[allow(dead_code)]
#[derive(Default, JsonSchema)]
struct MyStruct {
    #[schemars(skip_deserializing)]
    read_only: bool,
    #[schemars(skip_serializing)]
    write_only: bool,
    #[schemars(default)]
    default: bool,
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

// TODO enums
