use crate::prelude::*;
use schemars::generate::SchemaSettings;

#[derive(JsonSchema, Deserialize, Serialize)]
struct EmptyStruct {}

#[derive(JsonSchema, Deserialize, Serialize)]
enum External1 {
    Unit1,
    Unit2,
}

#[derive(JsonSchema, Deserialize, Serialize)]
enum External2 {
    Unit3,
    ValueNewType(Value),
}

#[derive(JsonSchema, Deserialize, Serialize)]
enum External3 {
    Int(u32),
    Tuple(u8, bool),
}

#[derive(JsonSchema, Deserialize, Serialize)]
enum External4 {
    StructNewType(EmptyStruct),
    Struct { foo: i32, bar: bool },
}

#[derive(JsonSchema, Deserialize, Serialize)]
struct ExternalContainer {
    f: f32,
    #[serde(flatten)]
    e1: External1,
    #[serde(flatten)]
    e2: External2,
    #[serde(flatten)]
    e3: External3,
    #[serde(flatten)]
    e4: External4,
}

impl ExternalContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: External1::Unit1,
                e2: External2::Unit3,
                e3: External3::Int(123),
                e4: External4::StructNewType(EmptyStruct {}),
            },
            Self {
                f: 9.87,
                e1: External1::Unit2,
                e2: External2::ValueNewType(json!({"key": "value"})),
                e3: External3::Tuple(0, true),
                e4: External4::Struct { foo: 1, bar: true },
            },
        ]
    }
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExternalContainerDenyUnknownFields {
    f: f32,
    #[serde(flatten)]
    e1: External1,
    #[serde(flatten)]
    e2: External2,
    #[serde(flatten)]
    e3: External3,
    #[serde(flatten)]
    e4: External4,
}

impl ExternalContainerDenyUnknownFields {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: External1::Unit1,
                e2: External2::Unit3,
                e3: External3::Int(123),
                e4: External4::StructNewType(EmptyStruct {}),
            },
            Self {
                f: 9.87,
                e1: External1::Unit2,
                e2: External2::ValueNewType(json!({"key": "value"})),
                e3: External3::Tuple(0, true),
                e4: External4::Struct { foo: 1, bar: true },
            },
        ]
    }
}

fn external_container_json_with_extra_field() -> Value {
    json!({
      "f": 1.23,
      "Unit1": null,
      "Unit3": null,
      "Int": 123,
      "StructNewType": {},
      "extra": null
    })
}

#[test]
fn external_enums_flattened() {
    test!(ExternalContainer)
        .assert_snapshot()
        .assert_allows_ser_roundtrip(ExternalContainer::values())
        .assert_matches_de_roundtrip(arbitrary_values())
        .assert_allows_de_roundtrip([external_container_json_with_extra_field()]);
}

#[test]
fn external_enums_flattened_deny_unknown_fields() {
    test!(ExternalContainerDenyUnknownFields)
        .assert_snapshot()
        .assert_allows_ser_roundtrip(ExternalContainerDenyUnknownFields::values())
        .assert_matches_de_roundtrip(arbitrary_values())
        .assert_rejects_de([external_container_json_with_extra_field()]);
}

#[test]
fn external_enums_flattened_deny_unknown_fields_draft07() {
    test!(
        ExternalContainerDenyUnknownFields,
        SchemaSettings::draft07()
    )
    .assert_snapshot()
    .assert_allows_ser_roundtrip(ExternalContainerDenyUnknownFields::values())
    .assert_matches_de_roundtrip(arbitrary_values())
    .assert_rejects_de([external_container_json_with_extra_field()]);
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(tag = "tag1")]
enum Internal1 {
    Unit1,
    Unit2,
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(tag = "tag2")]
enum Internal2 {
    Unit3,
    ValueNewType(Value),
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(tag = "tag3")]
enum Internal3 {
    StructNewType(EmptyStruct),
    Struct { foo: i32, bar: bool },
}

#[derive(JsonSchema, Deserialize, Serialize)]
struct InternalContainer {
    f: f32,
    #[serde(flatten)]
    e1: Internal1,
    #[serde(flatten)]
    e2: Internal2,
    #[serde(flatten)]
    e3: Internal3,
}

impl InternalContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: Internal1::Unit1,
                e2: Internal2::Unit3,
                e3: Internal3::StructNewType(EmptyStruct {}),
            },
            Self {
                f: 9.87,
                e1: Internal1::Unit2,
                e2: Internal2::ValueNewType(json!({"key": "value"})),
                e3: Internal3::Struct { foo: 1, bar: true },
            },
        ]
    }
}

#[test]
fn internal_enums_flattened() {
    test!(InternalContainer)
        .assert_snapshot()
        .assert_allows_ser_roundtrip(InternalContainer::values())
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(tag = "tag1", content = "content1")]
enum Adjacent1 {
    Unit1,
    Unit2,
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(tag = "tag2", content = "content2")]
enum Adjacent2 {
    Unit3,
    ValueNewType(Value),
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(tag = "tag3", content = "content3")]
enum Adjacent3 {
    Int(u32),
    Tuple(u8, bool),
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(tag = "tag4", content = "content4")]
enum Adjacent4 {
    StructNewType(EmptyStruct),
    Struct { foo: i32, bar: bool },
}

#[derive(JsonSchema, Deserialize, Serialize)]
struct AdjacentContainer {
    f: f32,
    #[serde(flatten)]
    e1: Adjacent1,
    #[serde(flatten)]
    e2: Adjacent2,
    #[serde(flatten)]
    e3: Adjacent3,
    #[serde(flatten)]
    e4: Adjacent4,
}

impl AdjacentContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: Adjacent1::Unit1,
                e2: Adjacent2::Unit3,
                e3: Adjacent3::Int(123),
                e4: Adjacent4::StructNewType(EmptyStruct {}),
            },
            Self {
                f: 9.87,
                e1: Adjacent1::Unit2,
                e2: Adjacent2::ValueNewType(json!({"key": "value"})),
                e3: Adjacent3::Tuple(0, true),
                e4: Adjacent4::Struct { foo: 1, bar: true },
            },
        ]
    }
}

#[test]
fn adjacent_enums_flattened() {
    test!(AdjacentContainer)
        .assert_snapshot()
        .assert_allows_ser_roundtrip(AdjacentContainer::values())
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(untagged)]
enum Untagged {
    Struct1 { foo: i32 },
    Struct2 { bar: bool },
    ValueNewType(Value),
}

#[derive(JsonSchema, Deserialize, Serialize)]
struct UntaggedContainer {
    f: f32,
    #[serde(flatten)]
    e1: Untagged,
}

impl UntaggedContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: Untagged::Struct1 { foo: 1 },
            },
            Self {
                f: 9.87,
                e1: Untagged::Struct2 { bar: true },
            },
            Self {
                f: 42.0,
                e1: Untagged::ValueNewType(json!({"key": "value"})),
            },
        ]
    }
}

#[test]
fn untagged_enums_flattened() {
    test!(UntaggedContainer)
        .assert_snapshot()
        .assert_allows_ser_roundtrip(UntaggedContainer::values())
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[derive(JsonSchema, Deserialize, Serialize)]
struct MixedContainer {
    f: f32,
    #[serde(flatten)]
    e1: External1,
    #[serde(flatten)]
    i2: Internal2,
    #[serde(flatten)]
    a3: Adjacent3,
    #[serde(flatten)]
    u: Untagged,
}

impl MixedContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: External1::Unit1,
                i2: Internal2::Unit3,
                a3: Adjacent3::Int(123),
                u: Untagged::Struct1 { foo: 1 },
            },
            Self {
                f: 9.87,
                e1: External1::Unit2,
                i2: Internal2::ValueNewType(json!({"key": "value"})),
                a3: Adjacent3::Tuple(0, true),
                u: Untagged::ValueNewType(json!({"key": "value"})),
            },
        ]
    }
}

#[test]
fn mixed_enums_flattened() {
    test!(MixedContainer)
        .assert_snapshot()
        .assert_allows_ser_roundtrip(MixedContainer::values())
        .assert_matches_de_roundtrip(arbitrary_values());
}
