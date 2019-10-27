/// Generates a [`Schema`](schema::Schema) for the given type using default settings.
///
/// The type must implement [`JsonSchema`].
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
}
