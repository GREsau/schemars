use crate::prelude::*;
use bytesize2::ByteSize;

#[test]
fn bytes() {
    test!(ByteSize)
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values());
}
