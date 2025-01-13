use crate::prelude::*;
use jiff01::civil::{Date, DateTime, Time};
use jiff01::{Timestamp, Zoned};

#[derive(JsonSchema, Serialize, Deserialize)]
struct JiffTypes {
    date_time_ts: Timestamp,
    date_time_zoned: Zoned,
    naive_date: Date,
    naive_date_time: DateTime,
    naive_time: Time,
}

#[test]
fn jiff() {
    test!(JiffTypes).assert_snapshot();

    test!(Timestamp)
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values());

    test!(Zoned)
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
}
