use schemars::schema::*;
use schemars::{gen, schema_for};
use serde_json::{from_str, to_string_pretty};
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_matches_default_settings() -> Result<(), Box<dyn std::error::Error>> {
        let expected_json = fs::read_to_string("tests/schema.json")?;
        let expected: Schema = from_str(&expected_json)?;

        let actual = schema_for!(Schema);
        fs::write("tests/schema.actual.json", to_string_pretty(&actual)?)?;

        assert_eq!(actual, expected, "\n\nGenerated schema did not match saved schema - generated schema has been written to \"tests/schema.actual.json\".");
        Ok(())
    }

    #[test]
    fn schema_matches_openapi3() -> Result<(), Box<dyn std::error::Error>> {
        let expected_json = fs::read_to_string("tests/schema-openapi3.json")?;
        let expected: Schema = from_str(&expected_json)?;

        let actual = gen::SchemaSettings::openapi3()
            .into_generator()
            .into_root_schema_for::<Schema>();
        fs::write(
            "tests/schema-openapi3.actual.json",
            to_string_pretty(&actual)?,
        )?;

        assert_eq!(actual, expected, "\n\nGenerated schema did not match saved schema - generated schema has been written to \"tests/schema-openapi3.actual.json\".");
        Ok(())
    }
}
