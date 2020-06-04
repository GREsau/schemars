use super::{Validate, ValidateMap, ValidateSeq, Validator};
use serde::{ser, ser::SerializeMap, Serialize};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    mem,
};

/// A span that is associated with values during validation.
///
/// This trait is automatically implemented.
pub trait Span: core::fmt::Debug + Clone {}
impl<T: core::fmt::Debug + Clone> Span for T {}

/// Spanner is used to provide spans for values that implement Serde Serialize.
///
/// A spanner is cloned for every nested value in a map or sequence,
/// so custom Clone implementation might be necessary.
pub trait Spanner: Clone {

    /// The span type that is associated with each value.
    type Span: Span;

    /// Span for a map key.
    fn key<S: ?Sized + Serialize>(&mut self, key: &S) -> Option<Self::Span>;

    /// Span for a value.
    fn value<S: ?Sized + Serialize>(&mut self, value: &S) -> Option<Self::Span>;

    /// Same as value but for unit types.
    fn unit(&mut self) -> Option<Self::Span>;

    /// Span for a map value.
    fn map_start(&mut self) -> Option<Self::Span>;

    /// Span for errors before closing a map.
    fn map_end(&mut self) -> Option<Self::Span>;

    /// Span for a sequence value.
    fn seq_start(&mut self) -> Option<Self::Span>;

    /// Span for errors before closing a sequence.
    fn seq_end(&mut self) -> Option<Self::Span>;
}

/// Spanned allows validation of any value that implements Serde Serialize with
/// a given [Spanner](Spanner).
pub struct Spanned<'k, S: ?Sized + Serialize, SP: Spanner> {
    spanner: SP,
    span: Option<SP::Span>,
    value: &'k S,
}

impl<'k, S, SP> Spanned<'k, S, SP>
where
    S: ?Sized + Serialize,
    SP: Spanner,
{
    /// Create a new spanned value.
    pub fn new(value: &'k S, spanner: SP) -> Self {
        Spanned {
            spanner,
            span: None,
            value,
        }
    }
}

impl<'k, S, SP> core::fmt::Display for Spanned<'k, S, SP>
where
    S: core::fmt::Display + ?Sized + Serialize,
    SP: Spanner,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<'k, S, SP> Hash for Spanned<'k, S, SP>
where
    S: ?Sized + Serialize + Hash,
    SP: Spanner,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

/// KeySpanner associates nested values with their
/// full path from the first value as a Vec of Strings.
///
/// Sequence indices are also turned into strings.
#[derive(Default)]
pub struct KeySpanner {
    keys: Vec<String>,

    // We use the same span for a map key and its value
    last_key: Option<String>,

    is_seq: bool,
    item_index: usize,
}

impl KeySpanner {
    /// Create a new Spanner.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Clone for KeySpanner {
    fn clone(&self) -> Self {
        let mut keys = self.keys.clone();
        if let Some(k) = &self.last_key {
            keys.push(k.clone());
        }

        KeySpanner {
            keys,
            last_key: None,
            is_seq: false,
            item_index: 0,
        }
    }
}

impl Spanner for KeySpanner {
    type Span = Vec<String>;

    fn key<S: ?Sized + Serialize>(&mut self, key: &S) -> Option<Self::Span> {
        let k = key.serialize(KeySerializer).unwrap();
        let mut keys = self.keys.clone();
        keys.push(k.clone());
        self.last_key = k.into();

        Some(keys)
    }

    fn value<S: ?Sized + Serialize>(&mut self, _value: &S) -> Option<Self::Span> {
        if self.is_seq {
            self.last_key = self.item_index.to_string().into();
            self.item_index += 1;
        }

        if let Some(k) = &self.last_key {
            let mut keys = self.keys.clone();
            keys.push(k.clone());
            return Some(keys);
        }

        Some(self.keys.clone())
    }

    fn unit(&mut self) -> Option<Self::Span> {
        if self.is_seq {
            self.last_key = self.item_index.to_string().into();
            self.item_index += 1;
        }

        if let Some(k) = mem::take(&mut self.last_key) {
            let mut keys = self.keys.clone();
            keys.push(k);
            return Some(keys);
        }

        Some(self.keys.clone())
    }

    fn map_start(&mut self) -> Option<Self::Span> {
        Some(self.keys.clone())
    }

    fn map_end(&mut self) -> Option<Self::Span> {
        Some(self.keys.clone())
    }

    fn seq_start(&mut self) -> Option<Self::Span> {
        self.is_seq = true;
        Some(self.keys.clone())
    }

    fn seq_end(&mut self) -> Option<Self::Span> {
        self.is_seq = false;
        Some(self.keys.clone())
    }
}

struct Hashed<'a, S: ?Sized + Serialize>(&'a S, u64);

impl<'a, S: ?Sized + Serialize> Hashed<'a, S> {
    fn new<H: Hasher>(value: &'a S, hasher: H) -> Self {
        Self(
            value,
            value.serialize(&mut HashSerializer { hasher }).unwrap(),
        )
    }
}

impl<'a, Ser: ?Sized + Serialize> Serialize for Hashed<'a, Ser> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'a, S: ?Sized + Serialize> Hash for Hashed<'a, S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.1);
    }
}

impl<'k, S, SP> Validate for Spanned<'k, S, SP>
where
    S: ?Sized + Serialize,
    SP: Spanner,
{
    type Span = SP::Span;

    fn validate<V: Validator<Self::Span>>(&self, validator: V) -> Result<(), V::Error> {
        let mut err = None;

        let k = SpannedInner {
            spanner: self.spanner.clone(),
            validator: Some(validator),
            validator_seq: None,
            validator_map: None,
            error: &mut err,
        };

        // We don't care about the serializer error,
        // all errors will be in "err".
        self.value.serialize(k).ok();

        match err {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }

    fn span(&self) -> Option<Self::Span> {
        self.span.clone()
    }
}

struct SpannedInner<'k, SP: Spanner, V: Validator<SP::Span>> {
    spanner: SP,

    validator: Option<V>,
    validator_seq: Option<V::ValidateSeq>,
    validator_map: Option<V::ValidateMap>,

    error: &'k mut Option<V::Error>,
}

impl<'k, SP: Spanner, V: Validator<SP::Span>> SpannedInner<'k, SP, V> {
    fn add_error(&mut self, e: V::Error) {
        match &mut self.error {
            Some(err) => {
                *err += e;
            }
            None => *self.error = Some(e),
        }
    }
}

/// A phantom type for the serializer
#[derive(Debug)]
struct SerdeError;

impl core::fmt::Display for SerdeError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for SerdeError {}

impl ser::Error for SerdeError {
    fn custom<T>(_msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        // This just not to cause panics,
        // but it is actually ignored
        SerdeError
    }
}

impl<'k, SP, V> ser::Serializer for SpannedInner<'k, SP, V>
where
    V: Validator<SP::Span>,
    SP: Spanner,
{
    type Ok = ();
    type Error = SerdeError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(mut self, v: bool) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_bool(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_i8(mut self, v: i8) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_i8(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_i16(mut self, v: i16) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_i16(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_i32(mut self, v: i32) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_i32(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_i64(mut self, v: i64) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_i64(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_u8(mut self, v: u8) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_u8(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_u16(mut self, v: u16) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_u16(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_u32(mut self, v: u32) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_u32(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_u64(mut self, v: u64) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_u64(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_f32(mut self, v: f32) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_f32(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_f64(mut self, v: f64) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_f64(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_char(mut self, v: char) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_char(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_str(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_bytes(mut self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(&v))
            .validate_bytes(v)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_none(mut self) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.unit())
            .validate_none()
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(mut self) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.unit())
            .validate_unit()
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_unit_struct(mut self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(name))
            .validate_unit_struct(name)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_unit_variant(
        mut self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        if let Err(e) = self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.value(variant))
            .validate_unit_variant(name, variant_index, variant)
        {
            self.add_error(e)
        }
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let mut m = self.serialize_map(Some(1))?;
        m.serialize_key(variant)?;
        m.serialize_value(value)?;
        m.end()
    }

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        match self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.seq_start())
            .validate_seq(len)
        {
            Ok(v) => {
                self.validator_seq = Some(v);
                Ok(self)
            }
            Err(e) => {
                self.add_error(e);
                Err(SerdeError)
            }
        }
    }

    fn serialize_tuple(mut self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        match self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.seq_start())
            .validate_seq(Some(len))
        {
            Ok(v) => {
                self.validator_seq = Some(v);
                Ok(self)
            }
            Err(e) => {
                self.add_error(e);
                Err(SerdeError)
            }
        }
    }

    fn serialize_tuple_struct(
        mut self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        match self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.seq_start())
            .validate_seq(Some(len))
        {
            Ok(v) => {
                self.validator_seq = Some(v);
                Ok(self)
            }
            Err(e) => {
                self.add_error(e);
                Err(SerdeError)
            }
        }
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(mut self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        match self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.map_start())
            .validate_map(len)
        {
            Ok(v) => {
                self.validator_map = Some(v);
                Ok(self)
            }
            Err(e) => {
                self.add_error(e);
                Err(SerdeError)
            }
        }
    }

    fn serialize_struct(
        mut self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        match self
            .validator
            .take()
            .unwrap()
            .with_span(self.spanner.map_start())
            .validate_map(Some(len))
        {
            Ok(v) => {
                self.validator_map = Some(v);
                Ok(self)
            }
            Err(e) => {
                self.add_error(e);
                Err(SerdeError)
            }
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

impl<'k, SP, V> ser::SerializeSeq for SpannedInner<'k, SP, V>
where
    V: Validator<SP::Span>,
    SP: Spanner,
{
    type Ok = ();
    type Error = SerdeError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let val: Hashed<T> = Hashed::new(value, DefaultHasher::new());

        let validator = self.validator_seq.as_mut().unwrap();

        let item_valid = validator.validate_element(&Spanned {
            span: self.spanner.value(value),
            spanner: self.spanner.clone(),
            value: &val,
        });

        if let Err(e) = item_valid {
            self.add_error(e);
        }

        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        let mut validator = self.validator_seq.take().unwrap();
        validator.set_span(self.spanner.seq_end());

        if let Err(e) = validator.end() {
            self.add_error(e);
        }

        Ok(())
    }
}

impl<'k, SP, V> ser::SerializeTuple for SpannedInner<'k, SP, V>
where
    V: Validator<SP::Span>,
    SP: Spanner,
{
    type Ok = ();
    type Error = SerdeError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

impl<'k, SP, V> ser::SerializeTupleStruct for SpannedInner<'k, SP, V>
where
    V: Validator<SP::Span>,
    SP: Spanner,
{
    type Ok = ();
    type Error = SerdeError;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

impl<'k, SP, V> ser::SerializeTupleVariant for SpannedInner<'k, SP, V>
where
    V: Validator<SP::Span>,
    SP: Spanner,
{
    type Ok = ();
    type Error = SerdeError;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'k, SP, V> ser::SerializeMap for SpannedInner<'k, SP, V>
where
    V: Validator<SP::Span>,
    SP: Spanner,
{
    type Ok = ();
    type Error = SerdeError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let k = key.serialize(KeySerializer).unwrap();

        let key_valid = self.validator_map.as_mut().unwrap().validate_key(&Spanned {
            spanner: self.spanner.clone(),
            span: self.spanner.key(key),
            value: &k,
        });

        if let Err(e) = key_valid {
            self.add_error(e);
        }

        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let valid = self
            .validator_map
            .as_mut()
            .unwrap()
            .validate_value(&Spanned {
                spanner: self.spanner.clone(),
                span: self.spanner.value(value),
                value,
            });

        if let Err(e) = valid {
            self.add_error(e);
        }

        Ok(())
    }

    fn end(mut self) -> Result<Self::Ok, Self::Error> {
        let mut validator = self.validator_map.take().unwrap();
        validator.set_span(self.spanner.map_end());

        if let Err(e) = validator.end() {
            self.add_error(e);
        }

        Ok(())
    }
}

impl<'k, SP, V> ser::SerializeStruct for SpannedInner<'k, SP, V>
where
    V: Validator<SP::Span>,
    SP: Spanner,
{
    type Ok = ();
    type Error = SerdeError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        <Self as ser::SerializeMap>::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as ser::SerializeMap>::end(self)
    }
}

impl<'k, SP, V> ser::SerializeStructVariant for SpannedInner<'k, SP, V>
where
    V: Validator<SP::Span>,
    SP: Spanner,
{
    type Ok = ();
    type Error = SerdeError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

/// Returned if a map key is not string, as json
/// only supports string keys.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct KeyNotStringError;

impl core::fmt::Display for KeyNotStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("keys must be strings")
    }
}

impl std::error::Error for KeyNotStringError {}

impl ser::Error for KeyNotStringError {
    fn custom<T>(_msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        // It is ignored.
        Self
    }
}

/// A serializer that only allows strings.
///
/// It converts integers to strings just like serde_json does.
struct KeySerializer;

impl ser::Serializer for KeySerializer {
    type Ok = String;
    type Error = KeyNotStringError;
    type SerializeSeq = ser::Impossible<String, KeyNotStringError>;
    type SerializeTuple = ser::Impossible<String, KeyNotStringError>;
    type SerializeTupleStruct = ser::Impossible<String, KeyNotStringError>;
    type SerializeTupleVariant = ser::Impossible<String, KeyNotStringError>;
    type SerializeMap = ser::Impossible<String, KeyNotStringError>;
    type SerializeStruct = ser::Impossible<String, KeyNotStringError>;
    type SerializeStructVariant = ser::Impossible<String, KeyNotStringError>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(KeyNotStringError)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(KeyNotStringError)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(KeyNotStringError)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(KeyNotStringError)
    }
}

/// A serializer that hashes a Serde Serialize value.
struct HashSerializer<H: Hasher> {
    hasher: H,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct ImpossibleError;

impl core::fmt::Display for ImpossibleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("this must not happen")
    }
}

impl std::error::Error for ImpossibleError {}

impl ser::Error for ImpossibleError {
    fn custom<T>(_msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        ImpossibleError
    }
}

impl<'h, H: Hasher> ser::Serializer for &'h mut HashSerializer<H> {
    type Ok = u64;
    type Error = ImpossibleError; 

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_u8(v as u8);
        Ok(self.hasher.finish())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_i8(v);
        Ok(self.hasher.finish())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_i16(v);
        Ok(self.hasher.finish())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_i32(v);
        Ok(self.hasher.finish())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_i64(v);
        Ok(self.hasher.finish())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_u8(v);
        Ok(self.hasher.finish())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_u16(v);
        Ok(self.hasher.finish())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_u32(v);
        Ok(self.hasher.finish())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_u64(v);
        Ok(self.hasher.finish())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.hasher.write(&v.to_le_bytes());
        Ok(self.hasher.finish())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.hasher.write(&v.to_le_bytes());
        Ok(self.hasher.finish())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.hasher.write_u32(v as u32);
        Ok(self.hasher.finish())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.hasher.write(v.as_bytes());
        Ok(self.hasher.finish())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.hasher.write(v);
        Ok(self.hasher.finish())
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.hasher.finish())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.hasher.finish())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.hasher.write(name.as_bytes());
        Ok(self.hasher.finish())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.hasher.write(variant.as_bytes());
        Ok(self.hasher.finish())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.hasher.write(variant.as_bytes());
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.hasher.write(variant.as_bytes());
        Ok(self)
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.hasher.write(variant.as_bytes());
        Ok(self)
    }
}

impl<'h, H: Hasher> ser::SerializeSeq for &'h mut HashSerializer<H> {
    type Ok = u64;
    type Error = ImpossibleError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.hasher.write_u8(1);
        value.serialize(&mut **self).ok();
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.hasher.finish())
    }
}

impl<'h, H: Hasher> ser::SerializeTuple for &'h mut HashSerializer<H> {
    type Ok = u64;
    type Error = ImpossibleError;
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.hasher.write_u8(1);
        value.serialize(&mut **self).ok();
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.hasher.finish())
    }
}

impl<'h, H: Hasher> ser::SerializeTupleVariant for &'h mut HashSerializer<H> {
    type Ok = u64;
    type Error = ImpossibleError;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.hasher.write_u8(1);
        value.serialize(&mut **self).ok();
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.hasher.finish())
    }
}

impl<'h, H: Hasher> ser::SerializeTupleStruct for &'h mut HashSerializer<H> {
    type Ok = u64;
    type Error = ImpossibleError;
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.hasher.write_u8(1);
        value.serialize(&mut **self).ok();
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.hasher.finish())
    }
}

impl<'h, H: Hasher> ser::SerializeStructVariant for &'h mut HashSerializer<H> {
    type Ok = u64;
    type Error = ImpossibleError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.hasher.write(key.as_bytes());
        value.serialize(&mut **self).ok();
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.hasher.finish())
    }
}

impl<'h, H: Hasher> ser::SerializeMap for &'h mut HashSerializer<H> {
    type Ok = u64;
    type Error = ImpossibleError;
    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut **self).ok();
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self).ok();
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.hasher.finish())
    }
}

impl<'h, H: Hasher> ser::SerializeStruct for &'h mut HashSerializer<H> {
    type Ok = u64;
    type Error = ImpossibleError;
    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.hasher.write(key.as_bytes());
        value.serialize(&mut **self).ok();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.hasher.finish())
    }
}
