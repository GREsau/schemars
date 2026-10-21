use crate::prelude::*;
use uuid1::Uuid;

const CASES: [Uuid; 3] = [Uuid::nil(), Uuid::max(), Uuid::from_u128(1234567890)];

#[test]
fn uuid() {
    test!(Uuid)
        .assert_snapshot()
        .assert_allows_ser_roundtrip(CASES)
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn braced_uuid() {
    test!(uuid1::fmt::Braced)
        .assert_snapshot()
        .assert_allows_ser_only(CASES.into_iter().map(uuid1::fmt::Braced::from_uuid));
}

#[test]
fn hyphenated_uuid() {
    test!(uuid1::fmt::Hyphenated)
        .assert_snapshot()
        .assert_allows_ser_only(CASES.into_iter().map(uuid1::fmt::Hyphenated::from_uuid));
}

#[test]
fn simple_uuid() {
    test!(uuid1::fmt::Simple)
        .assert_snapshot()
        .assert_allows_ser_only(CASES.into_iter().map(uuid1::fmt::Simple::from_uuid));
}

#[test]
fn urn_uuid() {
    test!(uuid1::fmt::Urn)
        .assert_snapshot()
        .assert_allows_ser_only(CASES.into_iter().map(uuid1::fmt::Urn::from_uuid));
}
