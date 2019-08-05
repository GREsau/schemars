use schemars::schema_for;
use schemars::schema::Schema;
use serde_json::Result;

fn main() -> Result<()> {
    let schema = schema_for!(Schema);
    let json = serde_json::to_string_pretty(&schema)?;
    println!("{}", json);

    Ok(())
}
