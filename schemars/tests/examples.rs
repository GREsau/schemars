mod util;
use std::ops::Range;

use schemars::JsonSchema;
use serde::Serialize;
use util::*;

#[derive(Default, Debug, JsonSchema, Serialize)]
#[schemars(example = "Struct::default", example = "null")]
pub struct Struct {
    #[schemars(example = "eight", example = "null")]
    foo: i32,
    bar: bool,
    #[schemars(example = "null")]
    baz: Option<&'static str>,
}

fn eight() -> i32 {
    8
}

fn null() -> () {}

#[test]
fn examples() -> TestResult {
    test_default_generated_schema::<Struct>("examples")
}

#[derive(Default, Debug, JsonSchema, Serialize)]
#[schemars(example = "Struct::default", examples = "array")]
pub struct StructExampleSet {
    #[schemars(examples = "range", example = "null")]
    foo: i32,
    #[schemars(examples = "bar_values")]
    bar: usize,
    #[schemars(examples = "array")]
    baz: Option<&'static str>,
}

fn range() -> Range<i32> {
    1..20
}

fn array() -> [&'static str; 3] {
    ["rust", "cpp", "c"]
}

fn bar_values() -> impl IntoIterator<Item = impl Serialize> {
    let mut u = 0u8;
    std::iter::from_fn(move || {
        u.checked_add(60).map(|v| {
            u = v;
            v
        })
    })
}

#[test]
fn example_sets() -> TestResult {
    test_default_generated_schema::<StructExampleSet>("example_sets")
}
