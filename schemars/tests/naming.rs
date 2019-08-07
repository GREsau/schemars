use schemars::MakeSchema;

mod util;
use util::*;

#[derive(MakeSchema)]
pub struct MyStruct<T, U, V, W> {
  pub t: T,
  pub u: U,
  pub v: V,
  pub w: W,
  pub inner: MySimpleStruct,
}

#[derive(MakeSchema)]
pub struct MySimpleStruct {}

#[test]
fn default_name_multiple_type_params() -> TestResult {
  test_default_generated_schema::<MyStruct<i32, (), bool, Vec<String>>>("naming-default")
}

#[derive(MakeSchema)]
#[serde(rename = "a-new-name-<W>-<T>-<T>")]
pub struct MyRenamedStruct<T, U, V, W> {
  pub t: T,
  pub u: U,
  pub v: V,
  pub w: W,
  pub inner: MySimpleRenamedStruct,
}

#[derive(MakeSchema)]
#[serde(rename = "another-new-name")]
pub struct MySimpleRenamedStruct {}

#[test]
#[ignore] // not yet implemented
fn overriden_with_rename_name_multiple_type_params() -> TestResult {
  test_default_generated_schema::<MyRenamedStruct<i32, (), bool, Vec<String>>>("naming-custom")
}
