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
    e1: Option<External1>,
    #[serde(flatten)]
    e2: Option<External2>,
    #[serde(flatten)]
    e3: Option<External3>,
    #[serde(flatten)]
    e4: Option<External4>,
}

impl ExternalContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: Some(External1::Unit1),
                e2: Some(External2::Unit3),
                e3: Some(External3::Int(123)),
                e4: Some(External4::StructNewType(EmptyStruct {})),
            },
            Self {
                f: 9.87,
                e1: Some(External1::Unit2),
                e2: Some(External2::ValueNewType(json!({"key": "value"}))),
                e3: Some(External3::Tuple(0, true)),
                e4: Some(External4::Struct { foo: 1, bar: true }),
            },
            Self {
                f: 9.87,
                e1: None,
                e2: None,
                e3: None,
                e4: None,
            },
        ]
    }
}

#[derive(JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExternalContainerDenyUnknownFields {
    f: f32,
    #[serde(flatten)]
    e1: Option<External1>,
    #[serde(flatten)]
    e2: Option<External2>,
    #[serde(flatten)]
    e3: Option<External3>,
    #[serde(flatten)]
    e4: Option<External4>,
}

impl ExternalContainerDenyUnknownFields {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: Some(External1::Unit1),
                e2: Some(External2::Unit3),
                e3: Some(External3::Int(123)),
                e4: Some(External4::StructNewType(EmptyStruct {})),
            },
            Self {
                f: 9.87,
                e1: Some(External1::Unit2),
                e2: Some(External2::ValueNewType(json!({"key": "value"}))),
                e3: Some(External3::Tuple(0, true)),
                e4: Some(External4::Struct { foo: 1, bar: true }),
            },
            Self {
                f: 9.87,
                e1: None,
                e2: None,
                e3: None,
                e4: None,
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
    e1: Option<Internal1>,
    #[serde(flatten)]
    e2: Option<Internal2>,
    #[serde(flatten)]
    e3: Option<Internal3>,
}

impl InternalContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: Some(Internal1::Unit1),
                e2: Some(Internal2::Unit3),
                e3: Some(Internal3::StructNewType(EmptyStruct {})),
            },
            Self {
                f: 9.87,
                e1: Some(Internal1::Unit2),
                e2: Some(Internal2::ValueNewType(json!({"key": "value"}))),
                e3: Some(Internal3::Struct { foo: 1, bar: true }),
            },
            Self {
                f: 9.87,
                e1: None,
                e2: None,
                e3: None,
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
    e1: Option<Adjacent1>,
    #[serde(flatten)]
    e2: Option<Adjacent2>,
    #[serde(flatten)]
    e3: Option<Adjacent3>,
    #[serde(flatten)]
    e4: Option<Adjacent4>,
}

impl AdjacentContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: Some(Adjacent1::Unit1),
                e2: Some(Adjacent2::Unit3),
                e3: Some(Adjacent3::Int(123)),
                e4: Some(Adjacent4::StructNewType(EmptyStruct {})),
            },
            Self {
                f: 9.87,
                e1: Some(Adjacent1::Unit2),
                e2: Some(Adjacent2::ValueNewType(json!({"key": "value"}))),
                e3: Some(Adjacent3::Tuple(0, true)),
                e4: Some(Adjacent4::Struct { foo: 1, bar: true }),
            },
            Self {
                f: 9.87,
                e1: None,
                e2: None,
                e3: None,
                e4: None,
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
    e1: Option<Untagged>,
}

impl UntaggedContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: Some(Untagged::Struct1 { foo: 1 }),
            },
            Self {
                f: 9.87,
                e1: Some(Untagged::Struct2 { bar: true }),
            },
            Self {
                f: 42.0,
                e1: Some(Untagged::ValueNewType(json!({"key": "value"}))),
            },
            Self { f: 42.0, e1: None },
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
    e1: Option<External1>,
    #[serde(flatten)]
    i2: Option<Internal2>,
    #[serde(flatten)]
    a3: Option<Adjacent3>,
    #[serde(flatten)]
    u: Option<Untagged>,
}

impl MixedContainer {
    fn values() -> impl IntoIterator<Item = Self> {
        [
            Self {
                f: 1.23,
                e1: Some(External1::Unit1),
                i2: Some(Internal2::Unit3),
                a3: Some(Adjacent3::Int(123)),
                u: Some(Untagged::Struct1 { foo: 1 }),
            },
            Self {
                f: 9.87,
                e1: Some(External1::Unit2),
                i2: Some(Internal2::ValueNewType(json!({"key": "value"}))),
                a3: Some(Adjacent3::Tuple(0, true)),
                u: Some(Untagged::ValueNewType(json!({"key": "value"}))),
            },
            Self {
                f: 9.87,
                e1: None,
                i2: None,
                a3: None,
                u: None,
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
