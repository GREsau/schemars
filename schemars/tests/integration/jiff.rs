macro_rules! jiff_tests {
    ($test_name:ident, $jiff:ident, $zoned:ident) => {
        #[test]
        fn $test_name() {
            use crate::prelude::*;
            use $jiff::civil::{Date, DateTime, Time};
            use $jiff::{$zoned, SignedDuration, Span, Timestamp};

            #[derive(JsonSchema, Serialize, Deserialize)]
            struct JiffTypes {
                date_time_ts: Timestamp,
                date_time_zoned: $zoned,
                naive_date: Date,
                naive_date_time: DateTime,
                naive_time: Time,
                duration: SignedDuration,
                span: Span,
            }

            test!(JiffTypes).assert_snapshot();

            test!(Timestamp)
                .assert_allows_ser_roundtrip_default()
                .assert_matches_de_roundtrip(arbitrary_values());

            test!($zoned)
                .assert_allows_ser_roundtrip_default()
                .assert_matches_de_roundtrip(arbitrary_values_except(
                    Value::is_string,
                    "Custom format 'zoned-date-time', so arbitrary strings technically allowed by schema",
                ));

            test!(Date)
                .assert_allows_ser_roundtrip_default()
                .assert_matches_de_roundtrip(arbitrary_values());

            test!(DateTime)
                .assert_allows_ser_roundtrip_default()
                .assert_matches_de_roundtrip(arbitrary_values_except(
                    Value::is_string,
                    "Custom format 'partial-date-time', so arbitrary strings technically allowed by schema",
                ));

            test!(Time)
                .assert_allows_ser_roundtrip_default()
                .assert_matches_de_roundtrip(arbitrary_values_except(
                    Value::is_string,
                    "Custom format 'date-time', so arbitrary strings technically allowed by schema",
                ));

            test!(SignedDuration)
                .assert_allows_ser_roundtrip_default()
                .assert_matches_de_roundtrip(arbitrary_values());

            test!(Span)
                .assert_allows_ser_roundtrip_default()
                .assert_matches_de_roundtrip(arbitrary_values());
        }
    };
}

#[cfg(feature = "jiff02")]
jiff_tests!(jiff02, jiff02, Zoned);

#[cfg(feature = "jiff1")]
jiff_tests!(jiff1, jiff1, ZonedDateTime);

#[cfg(all(feature = "jiff02", feature = "jiff1"))]
#[test]
fn jiff02_and_jiff1_identical() {
    use crate::prelude::*;

    test!(jiff1::Timestamp).assert_identical::<jiff02::Timestamp>();
    test!(jiff1::civil::Date).assert_identical::<jiff02::civil::Date>();
    test!(jiff1::civil::DateTime).assert_identical::<jiff02::civil::DateTime>();
    test!(jiff1::civil::Time).assert_identical::<jiff02::civil::Time>();
    test!(jiff1::SignedDuration).assert_identical::<jiff02::SignedDuration>();
    test!(jiff1::Span).assert_identical::<jiff02::Span>();
}
