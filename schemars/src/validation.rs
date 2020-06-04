use core::ops::AddAssign;
use std::hash::Hash;

pub mod span;
pub mod schema;

pub trait Validate {
    type Span: core::fmt::Debug + Clone;
    fn validate<V: Validator<Self::Span>>(&self, validator: V) -> Result<(), V::Error>;
    fn span(&self) -> Option<Self::Span>;
}

pub trait Validator<S: core::fmt::Debug + Clone>: Sized {
    type Error: std::error::Error + AddAssign;

    type ValidateSeq: ValidateSeq<S, Error = Self::Error>;
    type ValidateMap: ValidateMap<S, Error = Self::Error>;

    fn with_span(self, span: Option<S>) -> Self;

    fn validate_bool(self, v: bool) -> Result<(), Self::Error>;

    fn validate_i8(self, v: i8) -> Result<(), Self::Error>;
    fn validate_i16(self, v: i16) -> Result<(), Self::Error>;
    fn validate_i32(self, v: i32) -> Result<(), Self::Error>;
    fn validate_i64(self, v: i64) -> Result<(), Self::Error>;
    fn validate_i128(self, v: i128) -> Result<(), Self::Error>;

    fn validate_u8(self, v: u8) -> Result<(), Self::Error>;
    fn validate_u16(self, v: u16) -> Result<(), Self::Error>;
    fn validate_u32(self, v: u32) -> Result<(), Self::Error>;
    fn validate_u64(self, v: u64) -> Result<(), Self::Error>;
    fn validate_u128(self, v: u128) -> Result<(), Self::Error>;

    fn validate_f32(self, v: f32) -> Result<(), Self::Error>;
    fn validate_f64(self, v: f64) -> Result<(), Self::Error>;

    fn validate_char(self, v: char) -> Result<(), Self::Error>;
    fn validate_str(self, v: &str) -> Result<(), Self::Error>;

    fn validate_bytes(self, v: &[u8]) -> Result<(), Self::Error>;

    fn validate_none(self) -> Result<(), Self::Error>;
    fn validate_some<V: ?Sized>(self, value: &V) -> Result<(), Self::Error>
    where
        V: Validate<Span = S>;

    fn validate_unit(self) -> Result<(), Self::Error>;

    fn validate_unit_struct(self, name: &'static str) -> Result<(), Self::Error>;

    fn validate_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Self::Error>;

    fn validate_seq(self, len: Option<usize>) -> Result<Self::ValidateSeq, Self::Error>;

    fn validate_map(self, len: Option<usize>) -> Result<Self::ValidateMap, Self::Error>;
}

pub trait ValidateSeq<S: core::fmt::Debug + Clone> {
    type Error: std::error::Error;

    fn validate_element<V: ?Sized>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: Validate<Span = S> + Hash;

    fn end(self) -> Result<(), Self::Error>;
}

pub trait ValidateMap<S: core::fmt::Debug + Clone> {
    type Error: std::error::Error;

    // TODO: ToString fine here?
    fn validate_key<V: ?Sized>(&mut self, key: &V) -> Result<(), Self::Error>
    where
        V: Validate<Span = S> + ToString;

    fn validate_value<V: ?Sized>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: Validate<Span = S>;

    fn validate_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        // TODO: ToString fine here?
        K: Validate<Span = S> + ToString,
        V: Validate<Span = S>,
    {
        self.validate_key(key)?;
        self.validate_value(value)
    }

    fn end(self) -> Result<(), Self::Error>;
}
