#![allow(clippy::redundant_closure)] // serde_valid custom closure forms are intentional

use crate::prelude::*;
use serde_valid::Validate;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

fn always_ok(_val: &i32) -> Result<(), serde_valid::validation::Error> {
    Ok(())
}

fn always_ok_str(_val: &str) -> Result<(), serde_valid::validation::Error> {
    Ok(())
}

fn min_error_message(_params: &serde_valid::MinItemsError) -> String {
    "too few items".to_owned()
}

fn max_error_message(_params: &serde_valid::MaxItemsError) -> String {
    "too many items".to_owned()
}

fn sample_struct_rule(val: i32) -> Result<(), serde_valid::validation::Error> {
    if val >= 0 {
        Ok(())
    } else {
        Err(serde_valid::validation::Error::Custom(
            "must be non-negative".to_owned(),
        ))
    }
}

#[derive(JsonSchema, Deserialize, Serialize, Validate)]
#[validate(custom = |s| sample_struct_rule(s.min_max as i32))]
pub struct SerdeValidAttrStruct {
    // serde_valid: one validator per `#[validate(...)]` (not `minimum = 1, maximum = 100`)
    #[validate(minimum = 1.0)]
    #[validate(maximum = 100.0)]
    min_max: f32,
    #[validate(exclusive_minimum = 0.0)]
    #[validate(exclusive_maximum = 10.0)]
    exclusive_min_max: f32,
    #[validate(multiple_of = 2.5)]
    multiple: f32,
    #[validate(pattern = r"^[Hh]ello")]
    regex_str: String,
    #[validate(min_length = 1)]
    #[validate(max_length = 10)]
    non_empty_str: String,
    #[validate(min_items = 1)]
    #[validate(max_items = 3)]
    #[validate(unique_items)]
    items: Vec<i32>,
    // Value-level rules apply to each element of collections
    #[validate(min_length = 2)]
    item_lengths: Vec<String>,
    #[validate(minimum = 0)]
    #[validate(maximum = 10)]
    nested_numbers: Vec<i32>,
    #[validate(r#enum = [1, 2, 3])]
    enum_int: i32,
    #[validate(r#enum = ["a", "b"])]
    enum_str: String,
    #[validate(min_properties = 1)]
    #[validate(max_properties = 3)]
    props: HashMap<String, bool>,
    // Message variants (second slot only) — ignored for schema generation
    #[validate(min_items = 1, message_fn = min_error_message)]
    #[validate(max_items = 5, message_fn = max_error_message)]
    messaged_items: Vec<i32>,
    // Custom field validation — ignored for schema generation
    #[validate(custom = always_ok)]
    #[validate(custom = |v| always_ok(v))]
    custom_int: i32,
    // Nested dive
    #[validate]
    nested: SerdeValidNested,
}

#[derive(JsonSchema, Deserialize, Serialize, Validate, Default)]
pub struct SerdeValidNested {
    #[validate(minimum = -100)]
    #[validate(maximum = 100)]
    x: i32,
}

impl Default for SerdeValidAttrStruct {
    fn default() -> Self {
        Self {
            min_max: 1.0,
            exclusive_min_max: 5.0,
            multiple: 5.0,
            regex_str: "Hello world".to_owned(),
            non_empty_str: "test".to_owned(),
            items: vec![1, 2],
            item_lengths: vec!["ab".to_owned(), "cd".to_owned()],
            nested_numbers: vec![0, 10],
            enum_int: 1,
            enum_str: "a".to_owned(),
            props: HashMap::from([("x".to_owned(), true)]),
            messaged_items: vec![1],
            custom_int: 0,
            nested: SerdeValidNested { x: 0 },
        }
    }
}

impl SerdeValidAttrStruct {
    pub fn invalid_values() -> impl IntoIterator<Item = Self> {
        static MUTATORS: &[fn(&mut SerdeValidAttrStruct)] = &[
            |v| v.min_max = 0.9,
            |v| v.min_max = 100.1,
            |v| v.exclusive_min_max = 0.0,
            |v| v.exclusive_min_max = 10.0,
            |v| v.multiple = 3.0,
            |v| v.regex_str = "fail".to_owned(),
            |v| v.non_empty_str = String::new(),
            |v| v.items = Vec::new(),
            |v| v.items = vec![1, 2, 3, 4],
            |v| v.items = vec![1, 1],
            |v| v.item_lengths = vec!["a".to_owned()],
            |v| v.nested_numbers = vec![-1],
            |v| v.nested_numbers = vec![11],
            |v| v.enum_int = 4,
            |v| v.enum_str = "c".to_owned(),
            |v| v.props = HashMap::new(),
            |v| {
                v.props = HashMap::from([
                    ("a".to_owned(), true),
                    ("b".to_owned(), true),
                    ("c".to_owned(), true),
                    ("d".to_owned(), true),
                ])
            },
            |v| v.messaged_items = Vec::new(),
            |v| v.messaged_items = vec![1, 2, 3, 4, 5, 6],
            |v| v.nested.x = -101,
            |v| v.nested.x = 101,
        ];
        MUTATORS.iter().map(|f| {
            let mut result = SerdeValidAttrStruct::default();
            f(&mut result);
            result
        })
    }
}

#[test]
fn serde_valid_attrs() {
    test!(SerdeValidAttrStruct)
        .with_validator(|v| v.validate().is_ok())
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_rejects_invalid(SerdeValidAttrStruct::invalid_values())
        .assert_matches_de_roundtrip(arbitrary_values());
}

// Mirror serde_valid style: one keyword per `#[schemars(...)]` attribute.
#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(rename = "SerdeValidAttrStruct")]
pub struct SchemarsAttrStruct {
    #[schemars(minimum = 1.0)]
    #[schemars(maximum = 100.0)]
    min_max: f32,
    #[schemars(exclusive_minimum = 0.0)]
    #[schemars(exclusive_maximum = 10.0)]
    exclusive_min_max: f32,
    #[schemars(multiple_of = 2.5)]
    multiple: f32,
    #[schemars(pattern = r"^[Hh]ello")]
    regex_str: String,
    #[schemars(min_length = 1)]
    #[schemars(max_length = 10)]
    non_empty_str: String,
    #[schemars(min_items = 1)]
    #[schemars(max_items = 3)]
    #[schemars(unique_items)]
    items: Vec<i32>,
    #[schemars(min_length = 2)]
    item_lengths: Vec<String>,
    #[schemars(minimum = 0)]
    #[schemars(maximum = 10)]
    nested_numbers: Vec<i32>,
    #[schemars(r#enum = [1, 2, 3])]
    enum_int: i32,
    #[schemars(r#enum = ["a", "b"])]
    enum_str: String,
    #[schemars(min_properties = 1)]
    #[schemars(max_properties = 3)]
    props: HashMap<String, bool>,
    #[schemars(min_items = 1)]
    #[schemars(max_items = 5)]
    messaged_items: Vec<i32>,
    custom_int: i32,
    nested: SchemarsNested,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(rename = "SerdeValidNested")]
pub struct SchemarsNested {
    #[schemars(minimum = -100)]
    #[schemars(maximum = 100)]
    x: i32,
}

#[test]
fn schemars_attrs() {
    test!(SchemarsAttrStruct).assert_identical::<SerdeValidAttrStruct>();
}

#[derive(JsonSchema, Deserialize, Serialize, Validate)]
pub struct SerdeValidAttrTuple(
    #[validate(maximum = 10)] u8,
    #[validate(min_length = 1)] String,
);

#[test]
fn serde_valid_attrs_tuple() {
    test!(SerdeValidAttrTuple)
        .with_validator(|v| v.validate().is_ok())
        .assert_snapshot()
        .assert_allows_ser_roundtrip([SerdeValidAttrTuple(10, "a".to_owned())])
        .assert_rejects_invalid([
            SerdeValidAttrTuple(11, "a".to_owned()),
            SerdeValidAttrTuple(10, String::new()),
        ])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[derive(JsonSchema, Deserialize, Serialize, Validate)]
pub struct SerdeValidAttrNewType(#[validate(maximum = 10)] u8);

#[test]
fn serde_valid_attrs_newtype() {
    test!(SerdeValidAttrNewType)
        .with_validator(|v| v.validate().is_ok())
        .assert_snapshot()
        .assert_allows_ser_roundtrip([SerdeValidAttrNewType(10)])
        .assert_rejects_invalid([SerdeValidAttrNewType(11)])
        .assert_matches_de_roundtrip(arbitrary_values());
}

/// Custom validation forms that must compile and be ignored for schema output.
#[derive(JsonSchema, Deserialize, Serialize, Validate)]
#[validate(custom = |s| always_ok(&s.value))]
pub struct SerdeValidCustomForms {
    #[validate(custom = always_ok)]
    #[validate(custom(always_ok))]
    #[validate(custom = |v| always_ok(v))]
    value: i32,
    #[validate(minimum = 0, message = "too small")]
    #[validate(maximum = 10, message_fn = |_| "too large".to_owned())]
    bounded: i32,
    #[validate(pattern = r"^\w+$", message = "bad pattern")]
    #[validate(custom = always_ok_str)]
    named: String,
}

#[test]
fn serde_valid_custom_and_message_forms() {
    test!(SerdeValidCustomForms)
        .with_validator(|v| v.validate().is_ok())
        .assert_snapshot()
        .assert_allows_ser_roundtrip([SerdeValidCustomForms {
            value: 1,
            bounded: 5,
            named: "ok".to_owned(),
        }])
        .assert_rejects_invalid([
            SerdeValidCustomForms {
                value: 1,
                bounded: -1,
                named: "ok".to_owned(),
            },
            SerdeValidCustomForms {
                value: 1,
                bounded: 11,
                named: "ok".to_owned(),
            },
            SerdeValidCustomForms {
                value: 1,
                bounded: 5,
                named: "bad name".to_owned(),
            },
        ])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[derive(JsonSchema, Deserialize, Serialize, Validate, Clone)]
pub struct SerdeValidWrapperInner {
    #[validate(minimum = 1)]
    #[validate(maximum = 10)]
    n: i32,
}

/// Wrapper / pointer compositions supported by serde_valid composited validation.
#[allow(clippy::box_collection)] // intentionally tests `Box<Vec<_>>` composition
#[derive(JsonSchema, Deserialize, Serialize, Validate)]
pub struct SerdeValidWrapperFields {
    #[validate(minimum = 1)]
    #[validate(maximum = 10)]
    boxed: Box<i32>,
    #[validate(minimum = 1)]
    #[validate(maximum = 10)]
    rc: Rc<i32>,
    #[validate(minimum = 1)]
    #[validate(maximum = 10)]
    arc: Arc<i32>,
    #[validate(minimum = 1)]
    #[validate(maximum = 10)]
    optional: Option<i32>,
    #[validate(minimum = 1)]
    #[validate(maximum = 10)]
    optional_box: Option<Box<i32>>,
    #[validate(minimum = 1)]
    #[validate(maximum = 10)]
    box_optional: Box<Option<i32>>,
    #[validate(min_items = 1)]
    #[validate(max_items = 3)]
    #[allow(clippy::box_collection)] // intentionally tests `Box<Vec<_>>` composition
    boxed_vec: Box<Vec<i32>>,
    #[validate(min_length = 2)]
    rc_strings: Rc<Vec<String>>,
    #[validate]
    nested_box: Box<SerdeValidWrapperInner>,
    #[validate]
    nested_rc: Rc<SerdeValidWrapperInner>,
    #[validate]
    nested_optional_box: Option<Box<SerdeValidWrapperInner>>,
}

impl Default for SerdeValidWrapperFields {
    fn default() -> Self {
        Self {
            boxed: Box::new(5),
            rc: Rc::new(5),
            arc: Arc::new(5),
            optional: Some(5),
            optional_box: Some(Box::new(5)),
            box_optional: Box::new(Some(5)),
            boxed_vec: Box::new(vec![1, 2]),
            rc_strings: Rc::new(vec!["ab".to_owned()]),
            nested_box: Box::new(SerdeValidWrapperInner { n: 5 }),
            nested_rc: Rc::new(SerdeValidWrapperInner { n: 5 }),
            nested_optional_box: Some(Box::new(SerdeValidWrapperInner { n: 5 })),
        }
    }
}

impl SerdeValidWrapperFields {
    fn invalid_values() -> impl IntoIterator<Item = Self> {
        static MUTATORS: &[fn(&mut SerdeValidWrapperFields)] = &[
            |v| *v.boxed = 0,
            |v| v.rc = Rc::new(11),
            |v| v.arc = Arc::new(0),
            |v| v.optional = Some(0),
            |v| v.optional_box = Some(Box::new(11)),
            |v| *v.box_optional = Some(0),
            |v| *v.boxed_vec = Vec::new(),
            |v| *v.boxed_vec = vec![1, 2, 3, 4],
            |v| *Rc::make_mut(&mut v.rc_strings) = vec!["a".to_owned()],
            |v| v.nested_box.n = 0,
            |v| Rc::make_mut(&mut v.nested_rc).n = 11,
            |v| {
                if let Some(inner) = v.nested_optional_box.as_mut() {
                    inner.n = 0;
                }
            },
        ];
        MUTATORS.iter().map(|f| {
            let mut result = SerdeValidWrapperFields::default();
            f(&mut result);
            result
        })
    }
}

#[test]
fn serde_valid_wrapper_fields() {
    test!(SerdeValidWrapperFields)
        .with_validator(|v| v.validate().is_ok())
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_rejects_invalid(SerdeValidWrapperFields::invalid_values())
        .assert_matches_de_roundtrip(arbitrary_values());
}
