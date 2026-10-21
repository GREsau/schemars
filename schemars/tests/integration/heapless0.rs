use crate::prelude::*;

#[test]
fn heapless_vec() {
    test!(heapless0::Vec<usize, 10>)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([
            heapless0::Vec::new(),
            {
                let mut v = heapless0::Vec::new();
                v.push(1).ok();
                v.push(2).ok();
                v.push(3).ok();
                v.push(4).ok();
                v.push(5).ok();
                v
            },
        ])
        .assert_matches_de_roundtrip(
            (0..10).map(|len| serde_json::Value::Array((0..len).map(serde_json::Value::from).collect())),
        )
        .assert_matches_de_roundtrip(arbitrary_values_except(
            |v| matches!(v, serde_json::Value::Array(_)),
            "FIXME schema allows out-of-range positive integers",
        ));
}

#[test]
fn heapless_string() {
    test!(heapless0::String<16>)
        .assert_identical::<String>()
        .assert_allows_ser_roundtrip([
            heapless0::String::new(),
            {
                let mut s = heapless0::String::new();
                s.push_str("hello").ok();
                s
            },
        ])
        .assert_matches_de_roundtrip(arbitrary_values_except(
            serde_json::Value::is_string,
            "There's not a good way to express UTF-8 byte length in JSON schema, so schema ignores the String's capacity.",
        ));
}

#[test]
fn heapless_linear_map() {
    test!(heapless0::LinearMap<String, usize, 8>)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([heapless0::LinearMap::new()]);
}
