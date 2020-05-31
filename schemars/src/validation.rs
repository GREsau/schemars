#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_macros)]

use crate::{
    schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec},
    Set,
};
use serde::{ser, Serialize};
use std::{borrow::Cow, ops::Range};

/// A range where the value is, these are optionally
/// returned with validation errors.
///
/// The are no set rules, and how the span is interpreted
/// is up to the caller.
pub type Span = Range<u64>;

/// Spanned allows for providing spans for values during error reporting.
pub trait Spanned {
    fn span(&self) -> Option<Span>;
}

/// A Validator validates a serializable value.
pub trait Validator {
    type Error: std::error::Error;

    fn validate<T: ?Sized + Serialize>(&self, value: &T) -> Result<(), Self::Error>;
}

/// Validate is implemented for values that
/// know how to validate themselves.
pub trait Validate {
    type Error: std::error::Error;

    fn validate(&self) -> Result<(), Self::Error>;
}

/// An validation error with an optional span.
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    /// The span where the error ocurred.
    pub span: Option<Span>,

    /// The actual error details.
    pub value: ErrorValue,
}

impl Error {
    fn new(value: ErrorValue) -> Self {
        Self { span: None, value }
    }

    fn new_spanned(span: Span, value: ErrorValue) -> Self {
        Error {
            span: Some(span),
            value,
        }
    }
}

/// All the validation errors that can occur.
#[derive(Debug, Clone, PartialEq)]
// TODO maybe prefix or group them by type?
// TODO information about the schemas so that an error could display something like: "value is not a valid "SchemaName"" etc.
pub enum ErrorValue {
    /// Indicates a type that is not compatible with JSON.
    UnsupportedType,

    /// Indicates that the schema will never match any value.
    NotAllowed,

    /// Indicates invalid type.
    InvalidType { expected: SingleOrVec<InstanceType> },

    /// Indicates invalid enum value.
    InvalidEnumValue { expected: Vec<serde_json::Value> },

    /// Indicates that the number is not multiple of the given value.
    NotMultipleOf { multiple_of: f64 },

    /// Indicates that the number is less than the given minimum value.
    LessThanExpected { min: f64, exclusive: bool },

    /// Indicates that the number is more than the given maximum value.
    MoreThanExpected { max: f64, exclusive: bool },

    /// Indicates that the string doesn't match the given pattern.
    NoPatternMatch { pattern: String },

    /// Indicates that the string is too long.
    TooLong { max_length: u32 },

    /// Indicates that the string is too short.
    TooShort { min_length: u32 },

    /// Indicates that none of the subschemas matched.
    NoneValid { errors: Vec<Errors> },

    /// Indicates that more than one of the subschemas matched.
    MoreThanOneValid,

    /// Indicates that a not schema matched.
    ValidNot,

    /// Indicates that the items in the array are not unique.
    NotUnique,

    /// Indicates that the array doesn't contain the value of a given schema.
    MustContain,

    /// Indicates that the array doesn't have enough items.
    NotEnoughItems { min: usize },

    /// Indicates that the array has too many items.
    TooManyItems { max: usize },

    /// Indicates that the object has too few properties.
    NotEnoughProperties { min: usize },

    /// Indicates that the object has too many properties.
    TooManyProperties { max: usize },
}

// TODO vec impls
#[derive(Debug, Clone, PartialEq)]
/// This type is returned from validation.
pub struct Errors(Vec<Error>);

impl ser::Error for Errors {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        panic!("not supported")
    }
}

impl core::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Errors {}

impl Validator for RootSchema {
    type Error = Errors;

    fn validate<T: ?Sized + Serialize>(&self, value: &T) -> Result<(), Errors> {
        self.schema.validate(value)
    }
}

impl Validator for Schema {
    type Error = Errors;

    fn validate<T: ?Sized + Serialize>(&self, value: &T) -> Result<(), Errors> {
        match self {
            Schema::Bool(valid) => {
                if *valid {
                    Ok(())
                } else {
                    Err(Errors(vec![Error::new(ErrorValue::NotAllowed)]))
                }
            }
            Schema::Object(o) => o.validate(value),
        }
    }
}

impl Validator for SchemaObject {
    type Error = Errors;

    fn validate<T: ?Sized + Serialize>(&self, value: &T) -> Result<(), Errors> {
        todo!()
    }
}

struct SchemaValidator<'a> {
    errors: &'a mut Errors,
    schema: &'a SchemaObject,

    // Array tracking
    // They must be serialized unfortunately to check for uniqueness.
    //
    // We track the count separately, because we don't always need to
    // serialize them.
    arr_item_count: usize,
    arr_values: Vec<serde_json::Value>,
    arr_not_unique: bool,
    arr_contains: Option<&'a Schema>,

    // Object tracking
    obj_required: Set<String>,
    obj_prop_count: usize,
    obj_last_key: Option<String>,

    // For externally tagged variants
    first_variant: bool,
}

impl<'a> SchemaValidator<'a> {
    fn new(errors: &'a mut Errors, schema: &'a SchemaObject) -> Self {
        Self {
            errors,
            schema,
            arr_item_count: 0,
            arr_values: Vec::new(),
            arr_not_unique: false,
            arr_contains: None,
            obj_required: Set::new(),
            obj_prop_count: 0,
            obj_last_key: None,
            first_variant: true,
        }
    }
}

/// A phantom error type for the validator.
///
/// This is needed because we collect the errors in a vector
/// instead of returning them.
#[derive(Debug)]
struct SerdeError;

impl ser::Error for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        unreachable!()
    }
}

impl core::fmt::Display for SerdeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl std::error::Error for SerdeError {}

macro_rules! check_type {
    ($expected_type:ident, $schema:expr, $errors:expr) => {
        match &$schema.instance_type {
            Some(s) => match s {
                SingleOrVec::Single(single) => match &**single {
                    InstanceType::$expected_type => Ok(()),
                    _ => {
                        $errors.push(Error::new(ErrorValue::InvalidType {
                            expected: s.clone(),
                        }));
                        Err(SerdeError)
                    }
                },
                SingleOrVec::Vec(vec) => {
                    if vec.iter().any(|i| *i == InstanceType::$expected_type) {
                        Ok(())
                    } else {
                        $errors.push(Error::new(ErrorValue::InvalidType {
                            expected: s.clone(),
                        }));
                        Err(SerdeError)
                    }
                }
            },
            None => Err(SerdeError),
        }
    };
    (Object, $schema:expr, $errors:expr) => {
        match &$schema.instance_type {
            Some(s) => match s {
                SingleOrVec::Single(single) => match &**single {
                    InstanceType::Object => Ok(()),
                    _ => {
                        $errors.push(Error::new(ErrorValue::InvalidType {
                            expected: s.clone(),
                        }));
                        Err(SerdeError)
                    }
                },
                SingleOrVec::Vec(vec) => {
                    if vec.iter().any(|i| *i == InstanceType::Object) {
                        Ok(())
                    } else {
                        $errors.push(Error::new(ErrorValue::InvalidType {
                            expected: s.clone(),
                        }));
                        Err(SerdeError)
                    }
                }
            },
            None => Ok(()),
        }
    };
}

macro_rules! check_enum {
    (bool, $value:expr, $schema:expr, $errors:expr) => {
        if let Some(enum_vals) = &$schema.enum_values {
            let mut enum_contains = false;
            for val in enum_vals {
                if let Some(v) = val.as_bool() {
                    if v == $value {
                        enum_contains = true;
                        break;
                    }
                }
            }

            if enum_contains {
                Ok(())
            } else {
                $errors.push(Error::new(ErrorValue::InvalidEnumValue {
                    expected: enum_vals.clone(),
                }));
                Err(SerdeError)
            }
        } else {
            Ok(())
        }
    };
    (int, $value:expr, $schema:expr, $errors:expr) => {
        if let Some(enum_vals) = &$schema.enum_values {
            let mut enum_contains = false;
            for val in enum_vals {
                if let Some(v) = val.as_i64() {
                    if v == $value as i64 {
                        enum_contains = true;
                        break;
                    }
                }
                if let Some(v) = val.as_u64() {
                    if v == $value as u64 {
                        enum_contains = true;
                        break;
                    }
                }
            }

            if enum_contains {
                Ok(())
            } else {
                $errors.push(Error::new(ErrorValue::InvalidEnumValue {
                    expected: enum_vals.clone(),
                }));
                Err(SerdeError)
            }
        } else {
            Ok(())
        }
    };
    (float, $value:expr, $schema:expr, $errors:expr) => {
        if let Some(enum_vals) = &$schema.enum_values {
            let mut enum_contains = false;
            for val in enum_vals {
                if let Some(v) = val.as_f64() {
                    if v == $value as f64 {
                        enum_contains = true;
                        break;
                    }
                }
            }

            if enum_contains {
                Ok(())
            } else {
                $errors.push(Error::new(ErrorValue::InvalidEnumValue {
                    expected: enum_vals.clone(),
                }));
                Err(SerdeError)
            }
        } else {
            Ok(())
        }
    };
    (str, $value:expr, $schema:expr, $errors:expr) => {
        if let Some(enum_vals) = &$schema.enum_values {
            let mut enum_contains = false;
            for val in enum_vals {
                if let Some(v) = val.as_str() {
                    if v == $value {
                        enum_contains = true;
                        break;
                    }
                }
            }

            if enum_contains {
                Ok(())
            } else {
                $errors.push(Error::new(ErrorValue::InvalidEnumValue {
                    expected: enum_vals.clone(),
                }));
                Err(SerdeError)
            }
        } else {
            Ok(())
        }
    };
}

// TODO float conversions and equality
// TODO format
macro_rules! check_number {
    ($value:expr, $schema:expr, $errors:expr) => {
        if let Some(n) = &$schema.number {
            let mut number_err = false;

            if let Some(m) = n.multiple_of {
                if m != 0f64 && $value as f64 % m != 0f64 {
                    $errors.push(Error::new(ErrorValue::NotMultipleOf { multiple_of: m }));
                    number_err = true;
                }
            }

            if let Some(min) = n.minimum {
                if ($value as f64) < min {
                    $errors.push(Error::new(ErrorValue::LessThanExpected {
                        min,
                        exclusive: false,
                    }));
                    number_err = true;
                }
            }

            if let Some(min) = n.exclusive_minimum {
                if ($value as f64) <= min {
                    $errors.push(Error::new(ErrorValue::LessThanExpected {
                        min,
                        exclusive: true,
                    }));
                    number_err = true;
                }
            }

            if let Some(max) = n.maximum {
                if ($value as f64) > max {
                    $errors.push(Error::new(ErrorValue::MoreThanExpected {
                        max,
                        exclusive: false,
                    }));
                    number_err = true;
                }
            }

            if let Some(max) = n.exclusive_maximum {
                if ($value as f64) >= max {
                    $errors.push(Error::new(ErrorValue::MoreThanExpected {
                        max,
                        exclusive: true,
                    }));
                    number_err = true;
                }
            }
            if !number_err {
                Ok(())
            } else {
                Err(SerdeError)
            }
        } else {
            Ok(())
        };
    };
}

// TODO format
macro_rules! check_string {
    ($value:expr, $schema:expr, $errors:expr) => {
        if let Some(s) = &$schema.string {
            let mut string_err = false;

            if let Some(p) = &s.pattern {
                // TODO we assume that the regex in the schema is valid
                let re = regex::Regex::new(&*p).unwrap();
                if !re.is_match($value) {
                    $errors.push(Error::new(ErrorValue::NoPatternMatch {
                        pattern: p.clone(),
                    }));
                    string_err = true;
                }

                if let Some(max_length) = s.max_length {
                    if $value.chars().count() > max_length as usize {
                        $errors.push(Error::new(ErrorValue::TooLong { max_length }));
                        string_err = true;
                    }
                }

                if let Some(min_length) = s.min_length {
                    if $value.chars().count() < min_length as usize {
                        $errors.push(Error::new(ErrorValue::TooShort { min_length }));
                        string_err = true;
                    }
                }
            }

            if !string_err {
                Ok(())
            } else {
                Err(SerdeError)
            }
        } else {
            Ok(())
        }
    };
}

macro_rules! check_subschemas {
    ($value:expr, $schema:expr, $errors:expr) => {
        if let Some(sub) = &$schema.subschemas {
            let mut subschema_err = false;

            if let Some(all_of) = &sub.all_of {
                for s in all_of {
                    if let Err(e) = s.validate(&$value) {
                        $errors.extend(e.0.into_iter());
                        subschema_err = true;
                    }
                }
            }

            if let Some(any_of) = &sub.any_of {
                let mut validated = false;
                let mut errors: Vec<Errors> = Vec::with_capacity(any_of.len());
                for s in any_of {
                    match s.validate(&$value) {
                        Ok(_) => {
                            validated = true;
                            break;
                        }
                        Err(e) => {
                            errors.push(e);
                            subschema_err = true;
                        }
                    }
                }
                if !validated {
                    $errors.push(Error::new(ErrorValue::NoneValid { errors }));
                    subschema_err = true;
                }
            }

            if let Some(one_of) = &sub.one_of {
                let mut validated = false;
                let mut errors: Vec<Errors> = Vec::with_capacity(one_of.len());
                for s in one_of {
                    match s.validate(&$value) {
                        Ok(_) => {
                            if validated {
                                $errors.push(Error::new(ErrorValue::MoreThanOneValid));
                                subschema_err = true;
                                break;
                            }

                            validated = true;
                            break;
                        }
                        Err(e) => {
                            errors.push(e);
                            subschema_err = true;
                        }
                    }
                }
                if !validated {
                    $errors.push(Error::new(ErrorValue::NoneValid { errors }));
                    subschema_err = true;
                }
            }

            if let (Some(sub_if), Some(sub_then)) = (&sub.if_schema, &sub.then_schema) {
                if let Ok(_) = sub_if.validate(&$value) {
                    if let Err(e) = sub_then.validate(&$value) {
                        $errors.extend(e.0.into_iter());
                        subschema_err = true;
                    }
                } else {
                    if let Some(sub_else) = &sub.else_schema {
                        if let Err(e) = sub_else.validate(&$value) {
                            $errors.extend(e.0.into_iter());
                            subschema_err = true;
                        }
                    }
                }
            }

            if let Some(not) = &sub.not {
                if let Ok(_) = not.validate(&$value) {
                    $errors.push(Error::new(ErrorValue::ValidNot));
                    subschema_err = true;
                }
            }

            if !subschema_err {
                Ok(())
            } else {
                Err(SerdeError)
            }
        } else {
            Ok(())
        }
    };
}

impl<'a> ser::Serializer for SchemaValidator<'a> {
    type Ok = ();

    type Error = SerdeError;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<(), SerdeError> {
        check_type!(Boolean, &self.schema, &mut self.errors.0)?;
        check_enum!(bool, v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(int, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(int, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(int, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(int, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(int, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(int, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(int, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(int, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(float, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<(), SerdeError> {
        check_type!(Number, &self.schema, &mut self.errors.0)?;
        check_enum!(float, v, &self.schema, &mut self.errors.0)?;
        check_number!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<(), SerdeError> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<(), SerdeError> {
        check_type!(String, &self.schema, &mut self.errors.0)?;
        check_enum!(str, v, &self.schema, &mut self.errors.0)?;
        check_string!(v, &self.schema, &mut self.errors.0)?;
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<(), SerdeError> {
        check_type!(String, &self.schema, &mut self.errors.0)?;
        // TODO anything else to check here?
        check_subschemas!(v, &self.schema, &mut self.errors.0)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<(), SerdeError> {
        check_type!(Null, &self.schema, &mut self.errors.0)?;
        check_subschemas!(None as Option<()>, &self.schema, &mut self.errors.0)?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), SerdeError> {
        check_type!(Null, &self.schema, &mut self.errors.0)?;
        check_subschemas!(None as Option<()>, &self.schema, &mut self.errors.0)?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), SerdeError> {
        check_type!(Null, &self.schema, &mut self.errors.0)?;
        check_subschemas!(None as Option<()>, &self.schema, &mut self.errors.0)?;
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), SerdeError> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        let mut map_ser = self.serialize_map(Some(1)).unwrap();
        <Self as ser::SerializeMap>::serialize_entry(&mut map_ser, variant, value)?;
        <Self as ser::SerializeMap>::end(map_ser)?;
        Ok(())
    }

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq, SerdeError> {
        check_type!(Array, &self.schema, &mut self.errors.0)?;
        if let Some(arr) = &self.schema.array {
            if let Some(s) = &arr.contains {
                self.arr_contains = Some(s);
            }
        }
        if let Some(l) = len {
            self.arr_values.reserve(l);
        }
        Ok(self)
    }

    fn serialize_tuple(mut self, len: usize) -> Result<Self::SerializeTuple, SerdeError> {
        check_type!(Array, &self.schema, &mut self.errors.0)?;
        if let Some(arr) = &self.schema.array {
            if let Some(s) = &arr.contains {
                self.arr_contains = Some(s);
            }
        }

        self.arr_values.reserve(len);

        Ok(self)
    }

    fn serialize_tuple_struct(
        mut self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerdeError> {
        check_type!(Array, &self.schema, &mut self.errors.0)?;
        if let Some(arr) = &self.schema.array {
            if let Some(s) = &arr.contains {
                self.arr_contains = Some(s);
            }
        }

        self.arr_values.reserve(len);
        Ok(self)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        mut self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerdeError> {
        check_type!(Object, &self.schema, &mut self.errors.0)?;

        if let Some(obj) = &self.schema.object {
            self.obj_required = obj.required.clone();
        }

        <Self as ser::SerializeMap>::serialize_key(&mut self, variant)?;

        if let Some(obj) = &self.schema.object {
            if let Some(max) = obj.max_properties {
                if self.obj_prop_count > max as usize {
                    self.errors
                        .0
                        .push(Error::new(ErrorValue::TooManyProperties {
                            max: max as usize,
                        }))
                }
            }

            if let Some(min) = obj.min_properties {
                if self.obj_prop_count < min as usize {
                    self.errors
                        .0
                        .push(Error::new(ErrorValue::NotEnoughProperties {
                            min: min as usize,
                        }))
                }
            }
        }

        let s = get_variant_schema(variant, &self.schema);

        if let Some(s) = s {
            match s {
                Schema::Bool(false) => {
                    self.errors.0.push(Error::new(ErrorValue::NotAllowed));
                    return Err(SerdeError);
                }
                Schema::Bool(true) => {
                    // This is just for early return
                    return Err(SerdeError);
                }
                Schema::Object(o) => {
                    self.schema = o;
                }
            }
        }

        check_type!(Array, &self.schema, &mut self.errors.0)?;

        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(mut self, _len: Option<usize>) -> Result<Self::SerializeMap, SerdeError> {
        check_type!(Object, &self.schema, &mut self.errors.0)?;

        if let Some(obj) = &self.schema.object {
            self.obj_required = obj.required.clone();
        }

        Ok(self)
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(
        mut self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, SerdeError> {
        check_type!(Object, &self.schema, &mut self.errors.0)?;

        if let Some(obj) = &self.schema.object {
            self.obj_required = obj.required.clone();
        }

        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, SerdeError> {
        self.serialize_tuple_variant(name, variant_index, variant, len)
    }
}

impl<'a> ser::SerializeSeq for SchemaValidator<'a> {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        self.arr_item_count += 1;
        if let Some(arr) = &self.schema.array {
            if let Some(c) = self.arr_contains {
                if let Ok(_) = c.validate(value) {
                    self.arr_contains = None;
                }
            }

            if let Some(items) = &arr.items {
                match items {
                    SingleOrVec::Single(single_schema) => {
                        if let Err(e) = single_schema.validate(value) {
                            self.errors.0.extend(e.0.into_iter());
                        }
                    }
                    SingleOrVec::Vec(schemas) => {
                        if let Some(s) = schemas.get(self.arr_item_count - 1) {
                            if let Err(e) = s.validate(value) {
                                self.errors.0.extend(e.0.into_iter());
                            }
                        } else if let Some(s) = &arr.additional_items {
                            if let Err(e) = s.validate(value) {
                                self.errors.0.extend(e.0.into_iter());
                            }
                        }
                    }
                }
            }

            if let Some(true) = arr.unique_items {
                let item = serde_json::to_value(value).map_err(|e| {
                    self.errors.0.push(Error::new(ErrorValue::UnsupportedType));
                    SerdeError
                })?;

                if self.arr_values.iter().any(|v| &item == v) {
                    self.arr_not_unique = true;
                }
                self.arr_values.push(item);
            }
        }
        Ok(())
    }

    fn end(self) -> Result<(), SerdeError> {
        if let Some(c) = self.arr_contains {
            self.errors.0.push(Error::new(ErrorValue::MustContain));
        }

        if let Some(arr) = &self.schema.array {
            if let Some(min) = arr.min_items {
                if self.arr_item_count < min as usize {
                    self.errors
                        .0
                        .push(Error::new(ErrorValue::NotEnoughItems { min: min as usize }));
                }
            }

            if let Some(max) = arr.max_items {
                if self.arr_item_count > max as usize {
                    self.errors
                        .0
                        .push(Error::new(ErrorValue::TooManyItems { max: max as usize }));
                }
            }
        }

        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for SchemaValidator<'a> {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<(), SerdeError> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for SchemaValidator<'a> {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        <Self as ser::SerializeSeq>::serialize_element(self, value)
    }

    fn end(self) -> Result<(), SerdeError> {
        <Self as ser::SerializeSeq>::end(self)
    }
}

fn get_variant_schema<'s>(key: &str, schema: &'s SchemaObject) -> Option<&'s Schema> {
    if let Some(obj) = &schema.object {
        if let Some(prop_schema) = obj.properties.get(key) {
            return Some(prop_schema);
        }

        for (k, v) in obj.pattern_properties.iter() {
            // TODO we assume valid regex
            let key_re = regex::Regex::new(k).unwrap();
            if key_re.is_match(&key) {
                return Some(&v);
            }
        }

        if let Some(add_prop_schema) = &obj.additional_properties {
            return Some(add_prop_schema);
        }
        None
    } else {
        None
    }
}

impl<'a> ser::SerializeTupleVariant for SchemaValidator<'a> {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        <Self as ser::SerializeTuple>::serialize_element(self, value)
    }

    fn end(self) -> Result<(), SerdeError> {
        <Self as ser::SerializeTuple>::end(self)
    }
}

impl<'a> ser::SerializeMap for SchemaValidator<'a> {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        let s = key.serialize(KeySerializer).map_err(|e| {
            self.errors.0.push(Error::new(ErrorValue::UnsupportedType));
            SerdeError
        })?;

        self.obj_prop_count += 1;
        self.obj_required.remove(&s);

        if let Some(obj) = &self.schema.object {
            if let Some(name_schema) = &obj.property_names {
                if let Err(e) = name_schema.validate(&s) {
                    self.errors.0.extend(e.0.into_iter());
                    return Err(SerdeError);
                }
            }
        }

        self.obj_last_key = Some(s);

        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        let key = self.obj_last_key.take().expect("no key before value");

        if let Some(obj) = &self.schema.object {
            if let Some(prop_schema) = obj.properties.get(&key) {
                match prop_schema.validate(value) {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        self.errors.0.extend(e.0.into_iter());
                        return Err(SerdeError);
                    }
                }
            }

            for (k, v) in obj.pattern_properties.iter() {
                // TODO we assume valid regex
                let key_re = regex::Regex::new(k).unwrap();
                if key_re.is_match(&key) {
                    match v.validate(value) {
                        Ok(_) => {
                            return Ok(());
                        }
                        Err(e) => {
                            self.errors.0.extend(e.0.into_iter());
                            return Err(SerdeError);
                        }
                    }
                }
            }

            if let Some(add_prop_schema) = &obj.additional_properties {
                if let Err(e) = add_prop_schema.validate(value) {
                    self.errors.0.extend(e.0.into_iter());
                    return Err(SerdeError);
                }
            }
        }

        Ok(())
    }

    fn end(self) -> Result<(), SerdeError> {
        if let Some(obj) = &self.schema.object {
            if let Some(max) = obj.max_properties {
                if self.obj_prop_count > max as usize {
                    self.errors
                        .0
                        .push(Error::new(ErrorValue::TooManyProperties {
                            max: max as usize,
                        }))
                }
            }

            if let Some(min) = obj.min_properties {
                if self.obj_prop_count < min as usize {
                    self.errors
                        .0
                        .push(Error::new(ErrorValue::NotEnoughProperties {
                            min: min as usize,
                        }))
                }
            }
        }

        Ok(())
    }
}

impl<'a> ser::SerializeStruct for SchemaValidator<'a> {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        <Self as ser::SerializeMap>::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<(), SerdeError> {
        <Self as ser::SerializeMap>::end(self)
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for SchemaValidator<'a> {
    type Ok = ();
    type Error = SerdeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), SerdeError>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<(), SerdeError> {
        todo!()
    }
}

/// Returned if a map key is not string, as json
/// only supports string keys.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct KeyNotStringError;

impl core::fmt::Display for KeyNotStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("json keys must be strings")
    }
}

impl std::error::Error for KeyNotStringError {}

impl ser::Error for KeyNotStringError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        unreachable!()
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

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
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
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(KeyNotStringError)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(KeyNotStringError)
    }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(KeyNotStringError)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(KeyNotStringError)
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(KeyNotStringError)
    }
}
