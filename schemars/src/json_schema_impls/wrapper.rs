use crate::gen::SchemaGenerator;
use crate::schema::Schema;
use crate::JsonSchema;

macro_rules! deref_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: JsonSchema,
        {
            fn is_referenceable() -> bool {
                T::is_referenceable()
            }

            fn schema_name() -> String {
                T::schema_name()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                T::json_schema(gen)
            }

            fn json_schema_optional(gen: &mut SchemaGenerator) -> Schema {
                T::json_schema_optional(gen)
            }
        }
    };
}

deref_impl!(<'a, T: ?Sized> JsonSchema for &'a T);
deref_impl!(<'a, T: ?Sized> JsonSchema for &'a mut T);
deref_impl!(<T: ?Sized> JsonSchema for Box<T>);
deref_impl!(<T: ?Sized> JsonSchema for std::rc::Rc<T>);
deref_impl!(<T: ?Sized> JsonSchema for std::rc::Weak<T>);
deref_impl!(<T: ?Sized> JsonSchema for std::sync::Arc<T>);
deref_impl!(<T: ?Sized> JsonSchema for std::sync::Weak<T>);
deref_impl!(<T: ?Sized> JsonSchema for std::sync::Mutex<T>);
deref_impl!(<T: ?Sized> JsonSchema for std::sync::RwLock<T>);
deref_impl!(<T: ?Sized> JsonSchema for std::cell::Cell<T>);
deref_impl!(<T: ?Sized> JsonSchema for std::cell::RefCell<T>);
deref_impl!(<'a, T: ?Sized + ToOwned> JsonSchema for std::borrow::Cow<'a, T>);
deref_impl!(<T> JsonSchema for std::num::Wrapping<T>);
deref_impl!(<T> JsonSchema for std::cmp::Reverse<T>);
