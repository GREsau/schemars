use bigdecimal::BigDecimal;

use schemars::JsonSchema;
use util::*;

mod util;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct BigDecimalTypes {
    bigdecimal: BigDecimal,
}

#[test]
fn bigdecimal_types() -> TestResult {
    test_default_generated_schema::<BigDecimalTypes>("bigdecimal")
}
