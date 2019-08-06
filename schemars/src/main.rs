use schemars::{schema_for, schema::Schema};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let schema = schema_for!(Schema)?;
    let json = serde_json::to_string_pretty(&schema)?;
    println!("{}", json);

    Ok(())
}
