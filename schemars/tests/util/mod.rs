use pretty_assertions::assert_eq;
use schemars::{gen::SchemaSettings, schema_for, JsonSchema};
use std::error::Error;
use std::fs;
use std::panic;

pub type TestResult = Result<(), Box<dyn Error>>;

#[allow(dead_code)] // https://github.com/rust-lang/rust/issues/46379
pub fn test_generated_schema<T: JsonSchema>(file: &str, settings: SchemaSettings) -> TestResult {
    let expected_json = fs::read_to_string(format!("tests/expected/{}.json", file))?;
    let expected = serde_json::from_str(&expected_json)?;

    let actual = settings.into_generator().into_root_schema_for::<T>()?;

    if actual != expected {
        let actual_json = serde_json::to_string_pretty(&actual)?;
        fs::write(format!("tests/actual/{}.json", file), actual_json)?;
    }

    assert_eq!(actual, expected);
    Ok(())
}

#[allow(dead_code)] // https://github.com/rust-lang/rust/issues/46379
pub fn test_default_generated_schema<T: JsonSchema>(file: &str) -> TestResult {
    let expected_json = fs::read_to_string(format!("tests/expected/{}.json", file))?;
    let expected = serde_json::from_str(&expected_json)?;

    let actual = schema_for!(T)?;

    if actual != expected {
        let actual_json = serde_json::to_string_pretty(&actual)?;
        fs::write(format!("tests/actual/{}.json", file), actual_json)?;
    }

    assert_eq!(actual, expected);
    Ok(())
}
