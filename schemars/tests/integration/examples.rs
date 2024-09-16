use crate::prelude::*;

#[derive(Default, JsonSchema, Serialize)]
#[schemars(example = "Struct::default", example = "null")]
struct Struct {
    #[schemars(example = "eight", example = "null")]
    foo: i32,
    bar: bool,
    #[schemars(example = "null")]
    baz: Option<&'static str>,
}

fn eight() -> i32 {
    8
}

fn null() {}

#[test]
fn examples() {
    test!(Struct).assert_snapshot();
}
