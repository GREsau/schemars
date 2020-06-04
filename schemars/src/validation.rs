use core::ops::AddAssign;
use std::hash::Hash;

#[macro_use] mod macros;

pub mod schema;
pub mod span;

/// Validate is implemented by values that can be validate themselves 
/// against a given validator.
pub trait Validate {
    /// Span is information about the value.
    ///
    /// It is usually returned with the errors so that the caller knows
    /// where the error happened in case of nested values such as arrays or maps.
    type Span: span::Span;

    /// Validate self against a given validator.
    ///
    /// It is advised to call [with_span](Validator::with_span), or initialize
    /// the validator with a span, otherwise the [Validator](Validator)'s span will be empty.
    fn validate<V: Validator<Self::Span>>(&self, validator: V) -> Result<(), V::Error>;

    /// Return the span for the value if any.
    fn span(&self) -> Option<Self::Span>;
}

/// Values that implement [Validate](Validate) can validate themselves against
/// types that implement this trait.
///
/// It is modelled after Serde Serializer, and works in a very similar fashion.
pub trait Validator<S: core::fmt::Debug + Clone>: Sized {

    /// The error returned by the validator.
    ///
    /// The [AddAssign](AddAssign) bound is added so that some validators
    /// can return multiple errors together.
    type Error: std::error::Error + AddAssign;

    type ValidateSeq: ValidateSeq<S, Error = Self::Error>;
    type ValidateMap: ValidateMap<S, Error = Self::Error>;

    /// Set the span for the current value that is being validated.
    ///
    /// In some cases this is needed to ensure that the validator returns
    /// the correct span in its errors.
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

    fn set_span(&mut self, span: Option<S>);

    fn validate_element<V: ?Sized>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: Validate<Span = S> + Hash;

    fn end(self) -> Result<(), Self::Error>;
}

pub trait ValidateMap<S: core::fmt::Debug + Clone> {
    type Error: std::error::Error;

    fn set_span(&mut self, span: Option<S>);
    
    // TODO: Any way to get rid of the ToString bound here?
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
        // TODO: Any way to get rid of the ToString bound here?
        K: Validate<Span = S> + ToString,
        V: Validate<Span = S>,
    {
        self.validate_key(key)?;
        self.validate_value(value)
    }

    fn end(self) -> Result<(), Self::Error>;
}
