mod util;

use bson::oid::ObjectId;
use schemars::JsonSchema;
use url::Url;
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct BsonTypes {
    oid: ObjectId,
}

#[test]
fn bson_types() -> TestResult {
    test_default_generated_schema::<BsonTypes>("bson")
}
