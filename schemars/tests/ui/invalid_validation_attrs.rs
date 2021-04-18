use schemars::JsonSchema;

#[derive(JsonSchema)]
pub struct Struct1(#[validate(regex = 0, foo, length(min = 1, equal = 2, bar))] String);

#[derive(JsonSchema)]
pub struct Struct2(#[schemars(regex = 0, foo, length(min = 1, equal = 2, bar))] String);

fn main() {}
