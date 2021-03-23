/// Generates a [`RootSchema`](crate::schema::RootSchema) for the given type using default settings.
///
/// The type must implement [`JsonSchema`](crate::JsonSchema).
///
/// # Example
/// ```
/// use schemars::{schema_for, JsonSchema};
///
/// #[derive(JsonSchema)]
/// struct MyStruct {
///     foo: i32,
/// }
///
/// let my_schema = schema_for!(MyStruct);
/// ```
#[macro_export]
macro_rules! schema_for {
    ($type:ty) => {
        $crate::gen::SchemaGenerator::default().into_root_schema_for::<$type>()
    };
    ($_:expr) => {
        compile_error!("This argument to `schema_for!` is not a type - did you mean to use `schema_for_value!` instead?")
    };
}

// TODO document
#[macro_export]
macro_rules! schema_for_value {
    ($value:expr) => {
        $crate::gen::SchemaGenerator::default()
            .into_root_schema_for_value(&$value)
            .unwrap()
    };
}
