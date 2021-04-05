mod util;
use schemars::JsonSchema;
use util::*;

macro_rules! build_struct {
    (
        $id:ident { $($t:tt)* }
    ) => {
        #[derive(Debug, JsonSchema)]
        pub struct $id {
            x: u8,
            $($t)*
        }
    };
}

build_struct!(A { v: i32 });

#[test]
fn macro_built_struct() -> TestResult {
    test_default_generated_schema::<A>("macro_built_struct")
}
