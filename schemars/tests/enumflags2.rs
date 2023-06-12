mod util;

use schemars::JsonSchema_repr;
use enumflags2::{bitflags, BitFlags};
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
fn enum_state() -> TestResult {
    test_default_generated_schema::<EnumState>("enum-state")
}

#[test]
fn enum_bitflags_state() -> TestResult {
    test_default_generated_schema::<BitFlags<EnumState>>("enum-bitflags-state")
}
