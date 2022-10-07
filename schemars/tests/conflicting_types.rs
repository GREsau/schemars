mod util;
use schemars::JsonSchema;
use util::*;

mod mod_1 {
    use schemars::JsonSchema;

    #[derive(JsonSchema)]
    pub struct MyType {
        pub field: String,
    }
}

mod mod_2 {
    use schemars::JsonSchema;

    #[derive(JsonSchema)]
    pub enum MyType {
        StringNewType(String),
        StructVariant { floats: Vec<f32> },
    }
}

#[derive(JsonSchema)]
pub struct MyStruct {
    pub my_int: i32,
    pub my_bool: bool,
    pub my_nullable_enum1: Option<mod_1::MyType>,
    pub my_nullable_enum2: Option<mod_2::MyType>,
}

#[test]
fn transparent_struct() -> TestResult {
    test_default_generated_schema::<MyStruct>("conflicting_types")
}
