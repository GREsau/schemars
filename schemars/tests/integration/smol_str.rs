use crate::prelude::*;

#[cfg(feature = "smol_str02")]
#[test]
fn smol_str02() {
    test!(smol_str02::SmolStr)
        .assert_identical::<String>()
        .assert_allows_ser_roundtrip(["".into(), "test".into()])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[cfg(feature = "smol_str03")]
#[test]
fn smol_str03() {
    test!(smol_str03::SmolStr)
        .assert_identical::<String>()
        .assert_allows_ser_roundtrip(["".into(), "test".into()])
        .assert_matches_de_roundtrip(arbitrary_values());
}
