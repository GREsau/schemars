use pretty_assertions::assert_eq;
use schemars::visit::Visitor;
use schemars::{gen::SchemaSettings, schema::Schema, schema_for, JsonSchema};
use std::error::Error;
use std::fs;

pub type TestResult = Result<(), Box<dyn Error>>;

#[allow(dead_code)] // https://github.com/rust-lang/rust/issues/46379
pub fn test_generated_schema<T: JsonSchema>(file: &str, settings: SchemaSettings) -> TestResult {
    let actual = settings.into_generator().into_root_schema_for::<T>();
    test_schema(&actual, file)
}

#[allow(dead_code)] // https://github.com/rust-lang/rust/issues/46379
pub fn test_default_generated_schema<T: JsonSchema>(file: &str) -> TestResult {
    let actual = schema_for!(T);
    test_schema(&actual, file)
}

pub fn test_schema(actual: &Schema, file: &str) -> TestResult {
    // TEMP for easier comparison of schemas handling changes that don't actually affect a schema:
    // - `required` ordering has changed
    // - previously `f64` properties may now be integers
    let actual = &{
        let mut actual = actual.clone();
        TempFixupForTests.visit_schema(&mut actual);
        actual
    };

    let expected_json = match fs::read_to_string(format!("tests/expected/{}.json", file)) {
        Ok(j) => j,
        Err(e) => {
            write_actual_to_file(actual, file)?;
            return Err(Box::from(e));
        }
    };
    let expected = &serde_json::from_str(&expected_json)?;

    if actual != expected {
        write_actual_to_file(actual, file)?;
    }

    assert_eq!(expected, actual);
    Ok(())
}

fn write_actual_to_file(schema: &Schema, file: &str) -> TestResult {
    let actual_json = serde_json::to_string_pretty(&schema)?;
    fs::write(format!("tests/actual/{}.json", file), actual_json)?;
    Ok(())
}

struct TempFixupForTests;

impl schemars::visit::Visitor for TempFixupForTests {
    fn visit_schema(&mut self, schema: &mut Schema) {
        schemars::visit::visit_schema(self, schema);

        if let Some(object) = schema.as_object_mut() {
            if let Some(serde_json::Value::Array(required)) = object.get_mut("required") {
                required.sort_unstable_by(|a, b| a.as_str().cmp(&b.as_str()));
            }

            for (key, value) in object {
                if key == "multipleOf" || key.ends_with("aximum") || key.ends_with("inimum") {
                    if let Some(f) = value.as_f64() {
                        *value = f.into();
                    }
                }
            }
        }
    }
}
