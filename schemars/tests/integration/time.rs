use crate::prelude::*;
use time03::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset};

#[derive(JsonSchema, Serialize, Deserialize)]
struct TimeTypes {
    offset_date_time: OffsetDateTime,
    date: Date,
    primitive_date_time: PrimitiveDateTime,
    time: Time,
}

#[test]
fn time() {
    test!(TimeTypes).assert_snapshot();

    test!(OffsetDateTime)
        // .assert_allows_ser_roundtrip_default()
        // JSON Schema only allows dates with 4-digit years
        // .assert_allows_ser_roundtrip([OffsetDateTime::new_in_offset(Date::MIN, Time::MIDNIGHT, UtcOffset::UTC), OffsetDateTime::new_in_offset(Date::MAX, Time::MAX, UtcOffset::UTC)])
        .assert_matches_de_roundtrip(arbitrary_values());

    test!(Date)
        // JSON Schema only allows dates with 4-digit years
        // .assert_allows_ser_roundtrip([Date::MIN, Date::MAX])
        .assert_matches_de_roundtrip(arbitrary_values());

    test!(PrimitiveDateTime)
        // JSON Schema only allows dates with 4-digit years
        // .assert_allows_ser_roundtrip([PrimitiveDateTime::MIN, PrimitiveDateTime::MAX])
        .assert_matches_de_roundtrip(arbitrary_values_except(
            Value::is_string,
            "Custom format 'primitive-date-time', so arbitrary strings technically allowed by schema",
        ));

    test!(Time)
        // .assert_allows_ser_roundtrip([Time::MIDNIGHT, Time::MAX])
        .assert_matches_de_roundtrip(arbitrary_values_except(
            Value::is_string,
            "Custom format 'time', so arbitrary strings technically allowed by schema",
        ));

}
