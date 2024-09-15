use crate::prelude::*;
use schemars::{generate::SchemaSettings, Schema};

#[derive(JsonSchema, Deserialize, Serialize, Default)]
pub struct OuterStruct {
    #[schemars(extend("examples" = [8, null]))]
    maybe_int: Option<i32>,
    values: serde_json::Map<String, Value>,
    value: Value,
    inner: InnerEnum,
    maybe_inner: Option<InnerEnum>,
    tuples: Vec<(u8, i64)>,
}

#[derive(JsonSchema, Deserialize, Serialize, Default)]
pub enum InnerEnum {
    #[default]
    UndocumentedUnit1,
    UndocumentedUnit2,
    /// This is a documented unit variant
    DocumentedUnit,
    ValueNewType(Value),
}

#[test]
fn draft07() {
    test!(OuterStruct, SchemaSettings::draft07())
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn draft2019_09() {
    test!(OuterStruct, SchemaSettings::draft2019_09())
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn draft2020_12() {
    test!(OuterStruct, SchemaSettings::draft2020_12())
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn openapi3() {
    let mut settings = SchemaSettings::openapi3();
    // Hack to apply recursive transforms to schemas at components.schemas:
    // First, move them to $defs, then run the transforms, then move them back again.
    settings.transforms.insert(
        0,
        Box::new(|s: &mut Schema| {
            let obj = s.ensure_object();
            let defs = obj["components"]["schemas"].take();
            obj.insert("$defs".to_owned(), defs);
        }),
    );
    settings.transforms.push(Box::new(|s: &mut Schema| {
        let obj = s.ensure_object();
        obj["components"]["schemas"] = obj.remove("$defs").unwrap();
    }));

    test!(OuterStruct, settings).assert_snapshot();
}
