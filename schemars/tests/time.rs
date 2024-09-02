mod util;
use schemars::JsonSchema;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct TimeTypes {
    date: Date,
    offset_date_time: OffsetDateTime,
    primitive_date_time: PrimitiveDateTime,
    time: Time,
}

#[test]
fn time_types() -> TestResult {
    test_default_generated_schema::<TimeTypes>("time-types")
}
