use crate::prelude::*;

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(rename_all(serialize = "SCREAMING-KEBAB-CASE"), deny_unknown_fields)]
struct StructDenyUnknownFields {
    #[serde(skip_deserializing)]
    read_only: bool,
    #[allow(dead_code)]
    #[serde(skip_serializing)]
    write_only: bool,
    #[serde(default)]
    default: bool,
    #[serde(skip_serializing_if = "core::ops::Not::not")]
    skip_serializing_if: bool,
    #[serde(rename(serialize = "ser_renamed", deserialize = "de_renamed"))]
    renamed: bool,
    option: Option<bool>,
}

#[derive(JsonSchema, Deserialize, Serialize)]
struct StructAllowUnknownFields {
    #[serde(flatten)]
    inner: StructDenyUnknownFields,
}

#[test]
fn struct_deny_unknown_fields() {
    test!(StructDenyUnknownFields)
        .assert_snapshot()
        .assert_allows_de_roundtrip([
            json!({ "write_only": false, "skip_serializing_if": false, "de_renamed": false }),
            json!({ "write_only": true, "skip_serializing_if": true, "de_renamed": true, "default": true }),
            json!({ "write_only": true, "skip_serializing_if": true, "de_renamed": true, "option": true }),
        ])
        .assert_rejects_de([
            json!({ "skip_serializing_if": false, "de_renamed": false }),
            json!({ "write_only": false, "de_renamed": false }),
            json!({ "write_only": false, "skip_serializing_if": false }),
            json!({ "write_only": true, "skip_serializing_if": true, "de_renamed": true, "unknown": true }),
        ])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn struct_allow_unknown_fields() {
    test!(StructAllowUnknownFields)
        .assert_snapshot()
        .assert_allows_de_roundtrip([
            json!({ "write_only": false, "skip_serializing_if": false, "de_renamed": false }),
            json!({ "write_only": true, "skip_serializing_if": true, "de_renamed": true, "default": true }),
            json!({ "write_only": true, "skip_serializing_if": true, "de_renamed": true, "option": true }),
            json!({ "write_only": true, "skip_serializing_if": true, "de_renamed": true, "unknown": true }),
        ])
        .assert_rejects_de([
            json!({ "skip_serializing_if": false, "de_renamed": false }),
            json!({ "write_only": false, "de_renamed": false }),
            json!({ "write_only": false, "skip_serializing_if": false }),
        ])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[derive(JsonSchema, Deserialize, Serialize)]
struct TupleStruct(
    String,
    #[allow(dead_code)]
    #[serde(skip_serializing)]
    bool,
    String,
    #[serde(skip_deserializing)] bool,
    String,
);

#[test]
fn tuple_struct() {
    test!(TupleStruct)
        .assert_snapshot()
        .assert_allows_de_roundtrip([json!(["", true, "", ""])])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[allow(dead_code)]
#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(
    rename_all(serialize = "SCREAMING-KEBAB-CASE"),
    rename_all_fields(serialize = "PascalCase")
)]
enum ExternalEnum {
    #[serde(skip_deserializing)]
    ReadOnlyUnit,
    #[serde(skip_serializing)]
    WriteOnlyUnit,
    #[serde(skip_deserializing)]
    ReadOnlyStruct { s: String },
    #[serde(skip_serializing)]
    WriteOnlyStruct { i: isize },
    #[serde(rename(serialize = "ser_renamed_unit", deserialize = "de_renamed_unit"))]
    RenamedUnit,
    #[serde(rename(serialize = "ser_renamed_struct", deserialize = "de_renamed_struct"))]
    RenamedStruct { b: bool },
}

#[test]
fn externally_tagged_enum() {
    test!(ExternalEnum)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([
            ExternalEnum::ReadOnlyUnit,
            ExternalEnum::ReadOnlyStruct { s: "test".into() },
            ExternalEnum::RenamedUnit,
            ExternalEnum::RenamedStruct { b: true },
        ])
        .assert_allows_de_roundtrip([
            json!("WriteOnlyUnit"),
            json!({ "WriteOnlyStruct": { "i": 123 } }),
            json!("de_renamed_unit"),
            json!({ "de_renamed_struct": { "b": true } }),
        ])
        .assert_rejects_de([
            json!("READ-ONLY-UNIT"),
            json!("ReadOnlyUnit"),
            json!("ser_renamed_unit"),
        ])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[allow(dead_code)]
#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(
    tag = "tag",
    rename_all(serialize = "SCREAMING-KEBAB-CASE"),
    rename_all_fields(serialize = "PascalCase")
)]
enum InternalEnum {
    #[serde(skip_deserializing)]
    ReadOnlyUnit,
    #[serde(skip_serializing)]
    WriteOnlyUnit,
    #[serde(skip_deserializing)]
    ReadOnlyStruct { s: String },
    #[serde(skip_serializing)]
    WriteOnlyStruct { i: isize },
    #[serde(rename(serialize = "ser_renamed_unit", deserialize = "de_renamed_unit"))]
    RenamedUnit,
    #[serde(rename(serialize = "ser_renamed_struct", deserialize = "de_renamed_struct"))]
    RenamedStruct { b: bool },
}

#[test]
fn internally_tagged_enum() {
    test!(InternalEnum)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([
            InternalEnum::ReadOnlyUnit,
            InternalEnum::ReadOnlyStruct { s: "test".into() },
            InternalEnum::RenamedUnit,
            InternalEnum::RenamedStruct { b: true },
        ])
        .assert_allows_de_roundtrip([
            json!({ "tag": "WriteOnlyUnit" }),
            json!({ "tag": "WriteOnlyStruct", "i": 123 }),
            json!({ "tag": "de_renamed_unit" }),
            json!({ "tag": "de_renamed_struct", "b": true }),
        ])
        .assert_rejects_de([
            json!({ "tag": "READ-ONLY-UNIT" }),
            json!({ "tag": "ReadOnlyUnit" }),
            json!({ "tag": "ser_renamed_unit" }),
        ])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[allow(dead_code)]
#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(
    tag = "tag",
    content = "content",
    rename_all(serialize = "SCREAMING-KEBAB-CASE"),
    rename_all_fields(serialize = "PascalCase")
)]
enum AdjacentEnum {
    #[serde(skip_deserializing)]
    ReadOnlyUnit,
    #[serde(skip_serializing)]
    WriteOnlyUnit,
    #[serde(skip_deserializing)]
    ReadOnlyStruct { s: String },
    #[serde(skip_serializing)]
    WriteOnlyStruct { i: isize },
    #[serde(rename(serialize = "ser_renamed_unit", deserialize = "de_renamed_unit"))]
    RenamedUnit,
    #[serde(rename(serialize = "ser_renamed_struct", deserialize = "de_renamed_struct"))]
    RenamedStruct { b: bool },
}

#[test]
fn adjacently_tagged_enum() {
    test!(AdjacentEnum)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([
            AdjacentEnum::ReadOnlyUnit,
            AdjacentEnum::ReadOnlyStruct { s: "test".into() },
            AdjacentEnum::RenamedUnit,
            AdjacentEnum::RenamedStruct { b: true },
        ])
        .assert_allows_de_roundtrip([
            json!({ "tag": "WriteOnlyUnit" }),
            json!({ "tag": "WriteOnlyStruct", "content": { "i": 123 } }),
            json!({ "tag": "de_renamed_unit" }),
            json!({ "tag": "de_renamed_struct", "content": { "b": true } }),
        ])
        .assert_rejects_de([
            json!({ "tag": "READ-ONLY-UNIT" }),
            json!({ "tag": "ReadOnlyUnit" }),
            json!({ "tag": "ser_renamed_unit" }),
        ])
        .assert_matches_de_roundtrip(arbitrary_values());
}

fn false_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!(false)
}

#[allow(dead_code)]
#[derive(JsonSchema, Deserialize, Serialize, Default)]
struct WriteOnlyFalseSchema {
    #[serde(default, skip_serializing)]
    #[schemars(schema_with = "false_schema")]
    unsatisfiable: bool,
    #[serde(default, skip_serializing)]
    write_only: bool,
}

#[test]
fn write_only_false_schema() {
    // A `skip_serializing` field whose schema is `false` should remain the literal `false`
    // schema, with no vacuous `writeOnly` annotation, while a satisfiable `skip_serializing`
    // field is still annotated with `writeOnly`.
    test!(WriteOnlyFalseSchema)
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_allows_de_roundtrip([json!({}), json!({ "write_only": true })])
        .assert_matches_de_roundtrip(arbitrary_values_except(
            Value::is_array,
            "structs with `#derive(Deserialize)` can technically be deserialized from sequences, but that's not intended to be used via JSON, so schemars ignores it",
        ));
}

#[derive(JsonSchema, Deserialize, Serialize, Default)]
struct ReadOnlyFalseSchema {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    #[schemars(schema_with = "false_schema")]
    unsatisfiable: Option<bool>,
    #[serde(skip_deserializing)]
    read_only: bool,
}

#[test]
fn read_only_false_schema() {
    // The same applies to `readOnly` on `skip_deserializing` fields. The
    // `skip_serializing_if` attribute keeps the implied `default` value out of
    // the schema, so the `false` schema survives with no annotations at all.
    test!(ReadOnlyFalseSchema).assert_snapshot();
}

#[allow(dead_code)]
#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(
    untagged,
    rename_all(serialize = "SCREAMING-KEBAB-CASE"),
    rename_all_fields(serialize = "PascalCase")
)]
enum UntaggedEnum {
    #[serde(skip_deserializing)]
    ReadOnlyUnit,
    #[serde(skip_serializing)]
    WriteOnlyUnit,
    #[serde(skip_deserializing)]
    ReadOnlyStruct { s: String },
    #[serde(skip_serializing)]
    WriteOnlyStruct { i: isize },
    #[serde(rename(serialize = "ser_renamed_unit", deserialize = "de_renamed_unit"))]
    RenamedUnit,
    #[serde(rename(serialize = "ser_renamed_struct", deserialize = "de_renamed_struct"))]
    RenamedStruct { b: bool },
}

#[test]
fn untagged_enum() {
    test!(UntaggedEnum)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([
            UntaggedEnum::ReadOnlyUnit,
            UntaggedEnum::ReadOnlyStruct { s: "test".into() },
            UntaggedEnum::RenamedUnit,
            UntaggedEnum::RenamedStruct { b: true },
        ])
        .assert_allows_de_roundtrip([json!(null), json!({ "i": 123 }), json!({ "b": true })])
        .assert_rejects_de([json!({ "s": "test" })])
        .assert_matches_de_roundtrip(arbitrary_values());
}
