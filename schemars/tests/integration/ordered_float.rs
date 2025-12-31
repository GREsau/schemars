use ordered_float::OrderedFloat;

use crate::prelude::*;

#[test]
fn ordered_float() {
    test!(OrderedFloat<f64>)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([OrderedFloat(0.0), OrderedFloat(1.0)])
        .assert_matches_de_roundtrip(arbitrary_values());
}
