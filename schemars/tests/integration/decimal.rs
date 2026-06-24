use crate::prelude::*;

#[test]
fn decimal_types() {
    #[cfg(all(
        feature = "rust_decimal1",
        not(feature = "rust_decimal1_serde-float"),
        not(feature = "rust_decimal1_serde-arbitrary-precision")
    ))]
    test!(rust_decimal1::Decimal)
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values());

    #[cfg(feature = "bigdecimal04")]
    test!(bigdecimal04::BigDecimal)
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values());

    #[cfg(all(
        feature = "rust_decimal1",
        feature = "bigdecimal04",
        not(feature = "rust_decimal1_serde-float"),
        not(feature = "rust_decimal1_serde-arbitrary-precision")
    ))]
    test!(bigdecimal04::BigDecimal).assert_identical::<rust_decimal1::Decimal>();
}

#[test]
#[cfg(feature = "rust_decimal1_serde-float")]
fn rust_decimal_serde_float() {
    test!(rust_decimal1::Decimal)
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values_except(
            |v| v.is_string(),
            "serde-float only accepts numbers, not strings",
        ));
}

#[test]
#[cfg(all(
    feature = "rust_decimal1_serde-arbitrary-precision",
    not(feature = "rust_decimal1_serde-float")
))]
fn rust_decimal_serde_arbitrary_precision() {
    test!(rust_decimal1::Decimal)
        .assert_snapshot()
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values());
}
