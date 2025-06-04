use schemars::{schema_for, JsonSchema};

#[derive(JsonSchema)]
pub struct MyStruct<'a> {
    pub my_int: &'a i32,
    pub my_bool: bool,
    pub my_nullable_enum: Option<MyEnum>,
}

#[derive(JsonSchema)]
pub enum MyEnum {
    StringNewType(String),
    StructVariant { floats: Vec<f32> },
}

struct S<'a, 'b, T: 'a + 'b + ?Sized + ToOwned> {
    a: &'a T,
    b: &'b T,
}

impl<'a, 'b, T: 'a + 'b + ?Sized + ToOwned> JsonSchema for S<'a, 'b, T>
where
    for<'_schemars_derive> &'_schemars_derive T: JsonSchema,
    for<'_schemars_derive> &'_schemars_derive T: JsonSchema,
    for<'_schemars_derive> (): JsonSchema,
{
    fn schema_name() -> std::borrow::Cow<'static, str> {
        todo!()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        <&'a T>::schema_id() + <&'b T>::schema_id()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        todo!()
    }
}

fn schema_v<T: JsonSchema>(t: T) {}

fn main() {
    let a = "a".to_string();
    let s = S {
        a: a.as_str(),
        b: "",
    };

    schema_v(s);

    let schema = schema_for!(MyStruct);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
