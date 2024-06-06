mod util;
use schemars::JsonSchema;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct TimeTypes {
    date: Date,
    time: Time,
    primitive_date_time: PrimitiveDateTime,
    offset_date_time: OffsetDateTime,
}

#[test]
fn time_types() -> TestResult {
    test_default_generated_schema::<TimeTypes>("time-types")
}
