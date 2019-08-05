use schemars::*;
use serde_json::Result;

fn main() -> Result<()> {
    let gen = gen::SchemaGenerator::new();
    let schema = gen.into_root_schema_for::<Schema>();
    let json = serde_json::to_string_pretty(&schema)?;
    println!("{}", json);

    Ok(())
}
