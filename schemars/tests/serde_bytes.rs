mod util;
use serde_bytes::ByteBuf;
use util::*;

#[test]
fn bytes() -> TestResult {
    test_default_generated_schema::<(ByteBuf, ByteBuf)>("bytes")
}
