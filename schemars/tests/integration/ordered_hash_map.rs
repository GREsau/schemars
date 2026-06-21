use crate::prelude::*;
use ordered_hash_map06::{OrderedHashMap, OrderedHashSet};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn ordered_hash_map() {
    let mut map_with_entry = OrderedHashMap::new();
    map_with_entry.insert("key".to_owned(), true);

    test!(OrderedHashMap<String, bool>)
        .assert_identical::<BTreeMap<String, bool>>()
        .assert_allows_ser_roundtrip([
            OrderedHashMap::new(),
            map_with_entry,
        ])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn ordered_hash_set() {
    let mut set_with_entry = OrderedHashSet::new();
    set_with_entry.insert("test".to_owned());

    test!(OrderedHashSet<String>)
        .assert_identical::<BTreeSet<String>>()
        .assert_allows_ser_roundtrip([
            OrderedHashSet::new(),
            set_with_entry,
        ])
        .assert_matches_de_roundtrip(arbitrary_values());
}
