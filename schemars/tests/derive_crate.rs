use ::schemars as not_schemars;

#[allow(unused_imports)]
use std as schemars;

#[derive(Debug, not_schemars::JsonSchema)]
#[schemars(crate = "not_schemars")]
pub struct Struct {
    /// This is a document
    foo: i32,
    bar: bool,
}
