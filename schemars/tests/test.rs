use schemars::schema::*;
use schemars::schema_for;
use serde_json::{from_str, to_string_pretty};
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_matches() -> Result<(), Box<dyn std::error::Error>> {
        let expected_json = fs::read_to_string("tests/schema.json")?;
        let expected: Schema = from_str(&expected_json)?;

        let actual = schema_for!(Schema);
        fs::write("tests/schema.actual.json", to_string_pretty(&actual)?)?;

        assert_eq!(actual, expected, "\n\nGenerated schema did not match saved schema - generated schema has been written to \"tests/schema.actual.json\".");
        Ok(())
    }
}
