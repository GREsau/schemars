use schemars::{gen::SchemaSettings, JsonSchema};

/// # My Amazing Struct
///
///This struct shows off generating a schema with
/// a custom title and description.
///
/// Markdown input can be used without a hassle if we enable `raw_description_text` flag:
///
/// ```rust
/// let settings = SchemaSettings::draft07().with(|s| s.raw_description_text = true);
/// let mut generator = settings.into_generator();
/// let schema = generator.root_schema_for::<MyStruct>();
/// ```
#[derive(JsonSchema)]
pub struct MyStruct {
    /// # My Amazing Integer
    pub my_int: i32,
    /// This bool has a description, but no title.
    pub my_bool: bool,
    /// # A Nullable Enum
    /// This enum might be set, or it might not.
    pub my_nullable_enum: Option<MyEnum>,
}

/// # My Amazing Enum
#[derive(JsonSchema)]
pub enum MyEnum {
    /// A wrapper around a `String`
    StringNewType(String),
    /// A struct-like enum variant which contains
    /// some floats. T
    StructVariant {
        /// The floats themselves
        floats: Vec<f32>,
    },
}

fn main() {
    let settings = SchemaSettings::draft07().with(|s| s.raw_description_text = true);
    let mut generator = settings.into_generator();
    let schema = generator.root_schema_for::<MyStruct>();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
