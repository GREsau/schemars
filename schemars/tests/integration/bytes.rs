use crate::prelude::*;
use bytes1::{Bytes, BytesMut};

#[test]
fn bytes() {
    test!(Bytes)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([Bytes::new(), Bytes::from_iter([12; 34])])
        .assert_matches_de_roundtrip(arbitrary_values_except(
            is_array_of_u64,
            "FIXME schema allows out-of-range positive integers",
        ));
}

#[test]
fn bytes_mut() {
    test!(BytesMut).assert_identical::<Bytes>();
}

fn is_array_of_u64(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|a| a.iter().all(Value::is_u64))
}
