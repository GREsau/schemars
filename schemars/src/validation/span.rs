use super::{Validate, Validator};
use serde::{Serialize, Serializer};

pub struct Keys<'k, S: ?Sized + Serialize> {
    keys: Vec<String>,
    value: &'k S,
}

impl<'k, S: ?Sized + Serialize> Keys<'k, S> {
    pub fn new(value: &'k S) -> Self {
        Self {
            keys: Vec::new(),
            value,
        }
    }

    pub fn new_with_keys<K>(keys: K, value: &'k S) -> Self
    where
        K: IntoIterator,
        K::Item: AsRef<str>,
    {
        Self {
            keys: keys.into_iter().map(|s| s.as_ref().to_string()).collect(),
            value,
        }
    }
}

impl<'k, S: ?Sized + Serialize> Validate for Keys<'k, S> {
    type Span = Vec<String>;

    fn validate<V: Validator<Self::Span>>(&self, validator: V) -> Result<(), V::Error> {
        todo!()
    }

    fn span(&self) -> Option<Self::Span> {
        self.keys.clone().into()
    }
}

// struct KeysInner<S, V: Validator<S>> {
//     span: Option<S>,
//     validator: V,
// }

// impl<'k, S, V: Validator<S>> Serializer for &'k mut KeysInner<S, V> {

// }