mod util;

use schemars::JsonSchema_repr;
use enumflags2::bitflags;
use util::*;

#[derive(Copy, Clone, JsonSchema_repr)]
#[repr(u8)]
#[schemars(extension = "x-enumNames")]
#[bitflags]
pub enum EnumState {
    A,
    B,
    C,
    D
}

#[test]
fn enum_repr() -> TestResult {
    test_default_generated_schema::<EnumState>("enum-state")
}

