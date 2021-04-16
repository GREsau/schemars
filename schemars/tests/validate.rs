mod util;
use schemars::JsonSchema;
use std::collections::HashMap;
use util::*;

// In real code, this would typically be a Regex, potentially created in a `lazy_static!`.
static STARTS_WITH_HELLO: &'static str = r"^[Hh]ello\b";

#[derive(Debug, JsonSchema)]
pub struct Struct {
    #[validate(range(min = 0.01, max = 100))]
    min_max: f32,
    #[validate(regex = "STARTS_WITH_HELLO")]
    regex_str1: String,
    #[validate(regex(path = "STARTS_WITH_HELLO", code = "foo"))]
    regex_str2: String,
    #[validate(regex(pattern = r"^\d+$"))]
    regex_str3: String,
    #[validate(contains = "substring...")]
    contains_str1: String,
    #[validate(contains(pattern = "substring...", message = "bar"))]
    contains_str2: String,
    #[validate(email)]
    email_address: String,
    #[validate(phone)]
    tel: String,
    #[validate(url)]
    homepage: String,
    #[validate(length(min = 1, max = 100))]
    non_empty_str: String,
    #[validate(length(equal = 2))]
    pair: Vec<i32>,
    #[validate(contains = "map_key")]
    map_contains: HashMap<String, ()>,
    #[validate(required)]
    required_option: Option<bool>,
    #[validate(required)]
    #[serde(flatten)]
    required_flattened: Option<Inner>,
}

#[derive(Debug, JsonSchema)]
pub struct Inner {
    x: i32,
}

#[test]
fn validate() -> TestResult {
    test_default_generated_schema::<Struct>("validate")
}

#[derive(Debug, JsonSchema)]
pub struct Tuple(
    #[validate(range(max = 10))] u8,
    #[validate(required)] Option<bool>,
);

#[test]
fn validate_tuple() -> TestResult {
    test_default_generated_schema::<Tuple>("validate_tuple")
}

#[derive(Debug, JsonSchema)]
pub struct NewType(#[validate(range(max = 10))] u8);

#[test]
fn validate_newtype() -> TestResult {
    test_default_generated_schema::<NewType>("validate_newtype")
}
