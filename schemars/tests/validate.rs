mod util;
use schemars::JsonSchema;
use std::collections::BTreeMap;
use util::*;

struct FakeRegex(&'static str);

impl std::fmt::Display for FakeRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// In real code, this would typically be a Regex, potentially created in a `lazy_static!`.
static STARTS_WITH_HELLO: &FakeRegex = &FakeRegex(r"^[Hh]ello\b");

const MIN: u32 = 1;
const MAX: u32 = 1000;

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct Struct {
    #[validate(range(min = 0.01, max = 100))]
    min_max: f32,
    #[validate(range(min = "MIN", max = "MAX"))]
    min_max2: f32,
    #[validate(regex(path = *STARTS_WITH_HELLO))]
    regex_str1: String,
    #[validate(regex(path = "STARTS_WITH_HELLO", code = "foo"))]
    regex_str2: String,
    #[validate(contains(pattern = "substring..."))]
    contains_str1: String,
    #[validate(contains(pattern = "substring...", message = "bar"))]
    contains_str2: String,
    #[validate(email)]
    email_address: String,
    #[validate(url(code = "code_str", message = "message"))]
    homepage: String,
    #[validate(length(min = 1, max = 100))]
    non_empty_str: String,
    #[validate(length(min = "MIN", max = "MAX"))]
    non_empty_str2: String,
    #[validate(length(equal = 2))]
    pair: Vec<i32>,
    #[validate(contains(pattern = "map_key"))]
    map_contains: BTreeMap<String, ()>,
    #[validate(required)]
    required_option: Option<bool>,
    #[validate(required)]
    #[validate]
    #[serde(flatten)]
    required_flattened: Option<Inner>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct Inner {
    x: i32,
}

#[test]
fn validate() -> TestResult {
    test_default_generated_schema::<Struct>("validate")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct Struct2 {
    #[schemars(range(min = 0.01, max = 100))]
    min_max: f32,
    #[schemars(range(min = "MIN", max = "MAX"))]
    min_max2: f32,
    #[validate(regex(path = overridden))]
    #[schemars(regex(pattern = *STARTS_WITH_HELLO))]
    regex_str1: String,
    #[schemars(regex(pattern = r"^\d+$"))]
    regex_str2: String,
    #[validate(contains(pattern = "overridden"))]
    #[schemars(contains(pattern = "substring..."))]
    contains_str1: String,
    #[schemars(contains(pattern = "substring..."))]
    contains_str2: String,
    #[schemars(email)]
    email_address: String,
    #[schemars(url)]
    homepage: String,
    #[schemars(length(min = 1, max = 100))]
    non_empty_str: String,
    #[schemars(length(min = "MIN", max = "MAX"))]
    non_empty_str2: String,
    #[schemars(length(equal = 2))]
    pair: Vec<i32>,
    #[schemars(contains(pattern = "map_key"))]
    map_contains: BTreeMap<String, ()>,
    #[schemars(required)]
    required_option: Option<bool>,
    #[schemars(required)]
    #[serde(flatten)]
    required_flattened: Option<Inner>,
}

#[test]
fn validate_schemars_attrs() -> TestResult {
    test_default_generated_schema::<Struct2>("validate_schemars_attrs")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct Tuple(
    #[validate(range(max = 10))] u8,
    #[validate(required)] Option<bool>,
);

#[test]
fn validate_tuple() -> TestResult {
    test_default_generated_schema::<Tuple>("validate_tuple")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct NewType(#[validate(range(max = 10))] u8);

#[test]
fn validate_newtype() -> TestResult {
    test_default_generated_schema::<NewType>("validate_newtype")
}
