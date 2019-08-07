mod util;
use schemars::MakeSchema;
use util::*;

#[derive(Debug, MakeSchema)]
struct MyStruct<T, U, V, W> {
  t: T,
  u: U,
  v: V,
  w: W,
  inner: MySimpleStruct,
}

#[derive(Debug, MakeSchema)]
struct MySimpleStruct {}

#[test]
fn default_name_multiple_type_params() -> TestResult {
  test_default_generated_schema::<MyStruct<i32, (), bool, Vec<String>>>("naming-default")
}

#[derive(Debug, MakeSchema)]
#[serde(rename = "a-new-name-{W}-{T}-{T}")]
struct MyRenamedStruct<T, U, V, W> {
  t: T,
  u: U,
  v: V,
  w: W,
  inner: MySimpleRenamedStruct,
}

#[derive(Debug, MakeSchema)]
#[serde(rename = "another-new-name")]
struct MySimpleRenamedStruct {}

#[test]
fn overriden_with_rename_multiple_type_params() -> TestResult {
  test_default_generated_schema::<MyRenamedStruct<i32, (), bool, Vec<String>>>("naming-custom")
}
