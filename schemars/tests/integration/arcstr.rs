use arcstr1::ArcStr;

use crate::prelude::*;

#[test]
fn arcstr() {
    test!(ArcStr)
        .assert_identical::<String>()
        .assert_allows_ser_roundtrip(["".into(), "test".into()])
        .assert_matches_de_roundtrip(arbitrary_values());
}
