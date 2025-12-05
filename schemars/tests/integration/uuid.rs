use crate::prelude::*;
use uuid1::Uuid;

#[test]
fn uuid() {
    test!(Uuid)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([Uuid::nil(), Uuid::max(), Uuid::from_u128(1234567890)])
        .assert_matches_de_roundtrip(arbitrary_values());
}
