#[macro_export]
macro_rules! schema_for {
    ($type:path) => {
        $crate::gen::SchemaGenerator::default().into_root_schema_for::<$type>()
    };
}
