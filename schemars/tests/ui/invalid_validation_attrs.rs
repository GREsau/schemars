use schemars::JsonSchema;

// FIXME validation attrs like `email` should be disallowed non structs/enums/variants

#[derive(JsonSchema)]
#[validate(email)]
pub struct Struct1(#[validate(regex, foo, length(min = 1, equal = 2, bar))] String);

#[derive(JsonSchema)]
#[schemars(email)]
pub struct Struct2(#[schemars(regex, foo, length(min = 1, equal = 2, bar))] String);

#[derive(JsonSchema)]
pub struct Struct3(
    #[validate(
        regex = "foo",
        contains = "bar",
        regex(path = "baz"),
        phone,
        email,
        url
    )]
    String,
);

#[derive(JsonSchema)]
pub struct Struct4(
    #[schemars(
        regex = "foo",
        contains = "bar",
        regex(path = "baz"),
        phone,
        email,
        url
    )]
    String,
);

fn main() {}
