use crate::gen::SchemaGenerator;
use crate::{JsonSchema, Result};

macro_rules! deref_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: ?Sized + JsonSchema,
        {
            fn is_referenceable() -> bool {
                T::is_referenceable()
            }

            fn schema_name() -> String {
                T::schema_name()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Result {
                T::json_schema(gen)
            }
        }
    };
}

deref_impl!(<'a, T> JsonSchema for &'a T);
deref_impl!(<'a, T> JsonSchema for &'a mut T);
deref_impl!(<T> JsonSchema for Box<T>);
deref_impl!(<T> JsonSchema for std::rc::Rc<T>);
deref_impl!(<T> JsonSchema for std::sync::Arc<T>);
deref_impl!(<'a, T: ToOwned> JsonSchema for std::borrow::Cow<'a, T>);
