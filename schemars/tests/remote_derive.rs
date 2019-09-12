mod util;

use other_crate::Duration;
use schemars::JsonSchema;
use util::*;

mod other_crate {
    #[derive(Debug)]
    pub struct Duration {
        pub secs: i64,
        pub nanos: i32,
    }
}

#[derive(Debug, JsonSchema)]
#[serde(remote = "Duration")]
struct DurationDef {
    secs: i64,
    nanos: i32,
}

#[derive(Debug, JsonSchema)]
struct Process {
    command_line: String,
    #[serde(with = "DurationDef")]
    wall_time: Duration,
    #[serde(with = "DurationDef")]
    user_cpu_time: Duration,
    #[serde(deserialize_with = "some_serialize_function")]
    #[schemars(with = "DurationDef")]
    system_cpu_time: Duration,
}

#[test]
fn remote_derive_json_schema() -> TestResult {
    test_default_generated_schema::<Process>("remote_derive")
}
