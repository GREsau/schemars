use schemars::schema_for;
use schemars::JsonSchema;

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
    pub my_nullable_enum3: Option<mod_1::MyType>,
    pub my_nullable_enum4: Option<mod_2::MyType>,
}

fn main() {
    let schema = schema_for!(MyStruct);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
