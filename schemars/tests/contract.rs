mod util;
use schemars::{generate::SchemaSettings, JsonSchema};
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(rename_all(serialize = "SCREAMING-KEBAB-CASE"))]
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

#[allow(dead_code)]
#[derive(JsonSchema)]
struct TupleStruct(
    String,
    #[schemars(skip_serializing)] bool,
    String,
    #[schemars(skip_deserializing)] bool,
    String,
);

#[test]
fn contract_deserialize_tuple_struct() -> TestResult {
    test_generated_schema::<TupleStruct>(
        "contract_deserialize_tuple_struct",
        SchemaSettings::default().for_deserialize(),
    )
}

#[test]
fn contract_serialize_tuple_struct() -> TestResult {
    test_generated_schema::<TupleStruct>(
        "contract_serialize_tuple_struct",
        SchemaSettings::default().for_serialize(),
    )
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(
    rename_all(serialize = "SCREAMING-KEBAB-CASE"),
    rename_all_fields(serialize = "PascalCase")
)]
enum ExternalEnum {
    #[schemars(skip_deserializing)]
    ReadOnlyUnit,
    #[schemars(skip_serializing)]
    WriteOnlyUnit,
    #[schemars(skip_deserializing)]
    ReadOnlyStruct { s: String },
    #[schemars(skip_serializing)]
    WriteOnlyStruct { i: isize },
    #[schemars(rename(serialize = "ser_renamed_unit", deserialize = "de_renamed_unit"))]
    RenamedUnit,
    #[schemars(rename(serialize = "ser_renamed_struct", deserialize = "de_renamed_struct"))]
    RenamedStruct { b: bool },
}

#[test]
fn contract_deserialize_external_tag_enum() -> TestResult {
    test_generated_schema::<ExternalEnum>(
        "contract_deserialize_external_tag_enum",
        SchemaSettings::default().for_deserialize(),
    )
}

#[test]
fn contract_serialize_external_tag_enum() -> TestResult {
    test_generated_schema::<ExternalEnum>(
        "contract_serialize_external_tag_enum",
        SchemaSettings::default().for_serialize(),
    )
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(
    tag = "tag",
    rename_all(serialize = "SCREAMING-KEBAB-CASE"),
    rename_all_fields(serialize = "PascalCase")
)]
enum InternalEnum {
    #[schemars(skip_deserializing)]
    ReadOnlyUnit,
    #[schemars(skip_serializing)]
    WriteOnlyUnit,
    #[schemars(skip_deserializing)]
    ReadOnlyStruct { s: String },
    #[schemars(skip_serializing)]
    WriteOnlyStruct { i: isize },
    #[schemars(rename(serialize = "ser_renamed_unit", deserialize = "de_renamed_unit"))]
    RenamedUnit,
    #[schemars(rename(serialize = "ser_renamed_struct", deserialize = "de_renamed_struct"))]
    RenamedStruct { b: bool },
}

#[test]
fn contract_deserialize_internal_tag_enum() -> TestResult {
    test_generated_schema::<InternalEnum>(
        "contract_deserialize_internal_tag_enum",
        SchemaSettings::default().for_deserialize(),
    )
}

#[test]
fn contract_serialize_internal_tag_enum() -> TestResult {
    test_generated_schema::<InternalEnum>(
        "contract_serialize_internal_tag_enum",
        SchemaSettings::default().for_serialize(),
    )
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(
    tag = "tag",
    content = "content",
    rename_all(serialize = "SCREAMING-KEBAB-CASE"),
    rename_all_fields(serialize = "PascalCase")
)]
enum AdjacentEnum {
    #[schemars(skip_deserializing)]
    ReadOnlyUnit,
    #[schemars(skip_serializing)]
    WriteOnlyUnit,
    #[schemars(skip_deserializing)]
    ReadOnlyStruct { s: String },
    #[schemars(skip_serializing)]
    WriteOnlyStruct { i: isize },
    #[schemars(rename(serialize = "ser_renamed_unit", deserialize = "de_renamed_unit"))]
    RenamedUnit,
    #[schemars(rename(serialize = "ser_renamed_struct", deserialize = "de_renamed_struct"))]
    RenamedStruct { b: bool },
}

#[test]
fn contract_deserialize_adjacent_tag_enum() -> TestResult {
    test_generated_schema::<AdjacentEnum>(
        "contract_deserialize_adjacent_tag_enum",
        SchemaSettings::default().for_deserialize(),
    )
}

#[test]
fn contract_serialize_adjacent_tag_enum() -> TestResult {
    test_generated_schema::<AdjacentEnum>(
        "contract_serialize_adjacent_tag_enum",
        SchemaSettings::default().for_serialize(),
    )
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(
    untagged,
    rename_all(serialize = "SCREAMING-KEBAB-CASE"),
    rename_all_fields(serialize = "PascalCase")
)]
enum UntaggedEnum {
    #[schemars(skip_deserializing)]
    ReadOnlyUnit,
    #[schemars(skip_serializing)]
    WriteOnlyUnit,
    #[schemars(skip_deserializing)]
    ReadOnlyStruct { s: String },
    #[schemars(skip_serializing)]
    WriteOnlyStruct { i: isize },
    #[schemars(rename(serialize = "ser_renamed_unit", deserialize = "de_renamed_unit"))]
    RenamedUnit,
    #[schemars(rename(serialize = "ser_renamed_struct", deserialize = "de_renamed_struct"))]
    RenamedStruct { b: bool },
}

#[test]
fn contract_deserialize_untagged_enum() -> TestResult {
    test_generated_schema::<UntaggedEnum>(
        "contract_deserialize_untagged_enum",
        SchemaSettings::default().for_deserialize(),
    )
}

#[test]
fn contract_serialize_untagged_enum() -> TestResult {
    test_generated_schema::<UntaggedEnum>(
        "contract_serialize_untagged_enum",
        SchemaSettings::default().for_serialize(),
    )
}
