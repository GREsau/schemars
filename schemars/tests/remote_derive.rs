mod util;

use other_crate::Duration;
use schemars::JsonSchema;
use serde::Serialize;
use util::*;

mod other_crate {
    #[derive(Debug, Default)]
    pub struct Duration {
        pub secs: i64,
        pub nanos: i32,
    }
}

#[derive(Debug, JsonSchema, Serialize)]
#[serde(remote = "Duration")]
struct DurationDef {
    secs: i64,
    nanos: i32,
}

fn custom_serialize<S>(value: &Duration, ser: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    ser.collect_str(&format_args!("{}.{:09}s", value.secs, value.nanos))
}

#[derive(Debug, JsonSchema, Serialize)]
struct Process {
    command_line: String,
    #[serde(with = "DurationDef")]
    wall_time: Duration,
    #[serde(default, with = "DurationDef")]
    user_cpu_time: Duration,
    // FIXME this should serialize the default as "0.000000000s"
    #[serde(default, serialize_with = "custom_serialize")]
    #[schemars(with = "DurationDef")]
    system_cpu_time: Duration,
}

#[test]
fn remote_derive_json_schema() -> TestResult {
    test_default_generated_schema::<Process>("remote_derive")
}
