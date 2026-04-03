use crate::prelude::*;
use indexmap2::{IndexMap, IndexSet};
use ordermap::{ordermap, orderset, OrderMap, OrderSet};

#[test]
fn ordermap() {
    test!(OrderMap<String, bool>)
        .assert_identical::<IndexMap<String, bool>>()
        .assert_allows_ser_roundtrip([ordermap!(), ordermap!("key".to_owned() => true)])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn orderset() {
    test!(OrderSet<String>)
        .assert_identical::<IndexSet<String>>()
        .assert_allows_ser_roundtrip([orderset!(), orderset!("test".to_owned())])
        .assert_matches_de_roundtrip(arbitrary_values());
}
