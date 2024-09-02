mod util;
use schemars::JsonSchema;
use util::*;

const MIN: u32 = 1;
const MAX: u32 = 1000;

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct Struct {
    #[garde(range(min = 0.01, max = 100))]
    min_max: f32,
    #[garde(range(min = MIN, max = MAX))]
    min_max2: f32,
    #[garde(pattern(r"^[Hh]ello\b"))]
    regex_str1: String,
    #[garde(contains(concat!("sub","string...")))]
    contains_str1: String,
    #[garde(email)]
    email_address: String,
    #[garde(url)]
    homepage: String,
    #[garde(length(min = 1, max = 100))]
    non_empty_str: String,
    #[garde(length(min = MIN, max = MAX))]
    non_empty_str2: String,
    #[garde(length(equal = 2))]
    pair: Vec<i32>,
    #[garde(required)]
    required_option: Option<bool>,
    #[garde(required)]
    #[serde(flatten)]
    required_flattened: Option<Inner>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct Inner {
    x: i32,
}

#[test]
fn garde() -> TestResult {
    test_default_generated_schema::<Struct>("garde")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct Struct2 {
    #[schemars(range(min = 0.01, max = 100))]
    min_max: f32,
    #[schemars(range(min = MIN, max = MAX))]
    min_max2: f32,
    #[schemars(pattern(r"^[Hh]ello\b"))]
    regex_str1: String,
    #[schemars(contains(concat!("sub","string...")))]
    contains_str1: String,
    #[schemars(email)]
    email_address: String,
    #[schemars(url)]
    homepage: String,
    #[schemars(length(min = 1, max = 100))]
    non_empty_str: String,
    #[schemars(length(min = MIN, max = MAX))]
    non_empty_str2: String,
    #[schemars(length(equal = 2))]
    pair: Vec<i32>,
    #[schemars(required)]
    required_option: Option<bool>,
    #[schemars(required)]
    #[serde(flatten)]
    required_flattened: Option<Inner>,
}

#[test]
fn garde_schemars_attrs() -> TestResult {
    test_default_generated_schema::<Struct2>("garde_schemars_attrs")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct Tuple(
    #[garde(range(max = 10))] u8,
    #[garde(required)] Option<bool>,
);

#[test]
fn garde_tuple() -> TestResult {
    test_default_generated_schema::<Tuple>("garde_tuple")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct NewType(#[garde(range(max = 10))] u8);

#[test]
fn garde_newtype() -> TestResult {
    test_default_generated_schema::<NewType>("garde_newtype")
}
