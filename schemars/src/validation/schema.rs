//! Implementation of Validator for schemas.

use super::{span::Span, Validate, ValidateMap, ValidateSeq, Validator};
use crate::{
    schema::{InstanceType, Metadata, RootSchema, Schema, SchemaObject, SingleOrVec},
    Set,
};
use smallvec::SmallVec;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    ops::AddAssign,
};

/// An validation error with an optional span of the invalid value.
#[derive(Debug, Clone, PartialEq)]
pub struct Error<S: Span> {
    /// Information about the schema that caused the validation
    /// error.
    pub meta: Option<Box<Metadata>>,

    /// The span of the invalid value.
    pub span: Option<S>,

    /// The actual error details.
    pub value: ErrorValue<S>,
}

impl<S: Span> Error<S> {
    fn new(meta: Option<Box<Metadata>>, span: Option<S>, value: ErrorValue<S>) -> Self {
        Self { meta, span, value }
    }
}

impl<S: Span> core::fmt::Display for Error<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut start_paren = false;

        if let Some(span) = &self.span {
            write!(f, "({:?}", span)?;
            start_paren = true;
        }

        if f.alternate() {
            if let Some(meta) = &self.meta {
                if let Some(title) = &meta.title {
                    if start_paren {
                        write!(f, r#", schema: "{}""#, title)?;
                    } else {
                        write!(f, r#"(schema: "{}""#, title)?;
                    }
                }
            }
        }

        if start_paren {
            write!(f, ") ")?;
        }

        write!(f, "{}", self.value)
    }
}

/// All the validation errors that can occur.
#[derive(Debug, Clone, PartialEq)]
// TODO maybe prefix or group them by type?
pub enum ErrorValue<S: Span> {
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
    NoneValid { errors: Vec<Errors<S>> },

    /// Indicates that more than one of the subschemas matched.
    MoreThanOneValid { matched: Vec<Option<Box<Metadata>>> },

    /// Indicates that a not schema matched.
    ValidNot { matched: Option<Box<Metadata>> },

    /// Indicates that the items in the array are not unique.
    NotUnique {
        first: Option<S>,
        duplicate: Option<S>,
    },

    /// Indicates that the array doesn't contain the value of a given schema.
    MustContain { schema: Option<Box<Metadata>> },

    /// Indicates that the array doesn't have enough items.
    NotEnoughItems { min: usize },

    /// Indicates that the array has too many items.
    TooManyItems { max: usize },

    /// Indicates that the object has too few properties.
    NotEnoughProperties { min: usize },

    /// Indicates that the object has too many properties.
    TooManyProperties { max: usize },

    /// Indicates that a required property is missing.
    RequiredProperty { name: String },
}

impl<S: Span> core::fmt::Display for ErrorValue<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorValue::NotAllowed => write!(f, "value is not allowed here"),
            ErrorValue::InvalidType { expected } => write!(
                f,
                "invalid type, expected {}",
                match expected {
                    SingleOrVec::Single(s) => {
                        format!(r#""{:?}""#, s)
                    }
                    SingleOrVec::Vec(v) => {
                        let mut s = "one of ".into();

                        for (i, t) in v.iter().enumerate() {
                            s += format!(r#""{:?}""#, t).as_str();
                            if i != v.len() - 1 {
                                s += ", "
                            }
                        }

                        s
                    }
                }
            ),
            ErrorValue::InvalidEnumValue { expected } => {
                let enum_vals: Vec<String> = expected.iter().map(|v| v.to_string()).collect();
                write!(
                    f,
                    "invalid enum value, expected to be one of {}",
                    enum_vals.join(", ")
                )
            }
            ErrorValue::NotMultipleOf { multiple_of } => {
                write!(f, "the value is expected to be multiple of {}", multiple_of)
            }
            ErrorValue::LessThanExpected { min, exclusive } => {
                if *exclusive {
                    write!(f, "the value is expected to be more than {}", min)
                } else {
                    write!(f, "the value is expected to be at least {}", min)
                }
            }
            ErrorValue::MoreThanExpected { max, exclusive } => {
                if *exclusive {
                    write!(f, "the value is expected to be less than {}", max)
                } else {
                    write!(f, "the value is expected to be at most {}", max)
                }
            }
            ErrorValue::NoPatternMatch { pattern } => {
                write!(f, r#"the string must match the pattern "{}""#, pattern)
            }
            ErrorValue::TooLong { max_length } => write!(
                f,
                r#"the string must not be longer than {} characters"#,
                max_length
            ),
            ErrorValue::TooShort { min_length } => write!(
                f,
                r#"the string must must be at least {} characters long"#,
                min_length
            ),
            ErrorValue::NoneValid { errors } => {
                writeln!(f, r#"no subschema matched the value:"#)?;

                for (i, e) in errors.iter().enumerate() {
                    write!(f, "{}", e)?;

                    if i != errors.len() - 1 {
                        writeln!(f, "\n")?;
                    }
                }

                Ok(())
            }
            ErrorValue::MoreThanOneValid { matched } => writeln!(
                f,
                r#"expected exactly one schema to match, but {} schemas matched"#,
                matched.len()
            ),
            ErrorValue::ValidNot { matched } => {
                if let Some(meta) = matched {
                    if let Some(title) = &meta.title {
                        return writeln!(f, r#"the value must not be a "{}""#, title);
                    }
                }

                writeln!(f, r#"the value is disallowed by a "not" schema"#)
            }
            ErrorValue::NotUnique { first, duplicate } => {
                if let (Some(first), Some(dup)) = (first, duplicate) {
                    writeln!(
                        f,
                        r#"all items in the array must be unique, but "{:?}" and "{:?}" are the same"#,
                        first, dup
                    )
                } else {
                    writeln!(f, r#"all items in the array must be unique"#)
                }
            }
            ErrorValue::MustContain { schema } => {
                if let Some(meta) = schema {
                    if let Some(title) = &meta.title {
                        return writeln!(
                            f,
                            r#"at least one of the items in the array must be "{}""#,
                            title
                        );
                    }
                }

                writeln!(
                    f,
                    r#"at least one of the items in the array must match the given schema"#
                )
            }
            ErrorValue::NotEnoughItems { min } => {
                write!(f, "the array must have at least {} items", min)
            }
            ErrorValue::TooManyItems { max } => {
                write!(f, "the array cannot have more than {} items", max)
            }
            ErrorValue::NotEnoughProperties { min } => {
                write!(f, "the object must have at least {} properties", min)
            }
            ErrorValue::TooManyProperties { max } => {
                write!(f, "the object cannot have more than {} properties", max)
            }
            ErrorValue::RequiredProperty { name } => {
                write!(f, r#"the required property "{}" is missing"#, name)
            }
        }
    }
}

type SmallVecArray<S> = [Error<S>; 10];

/// In a lot of cases there are only 1 or 2 errors.
/// I'm not sure whether this makes the overall performance better or not.
type ErrorsInner<S> = SmallVec<SmallVecArray<S>>;

/// This type is returned from validation.
#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct Errors<S: Span>(ErrorsInner<S>);

impl<S: Span> Errors<S> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Error<S>> {
        self.0.iter()
    }
}

impl<S: Span> IntoIterator for Errors<S> {
    type Item = <ErrorsInner<S> as IntoIterator>::Item;

    type IntoIter = smallvec::IntoIter<SmallVecArray<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<S: Span> Errors<S> {
    fn new() -> Self {
        Errors(SmallVec::new())
    }
}

impl<S: Span> core::fmt::Display for Errors<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.0 {
            writeln!(f, "{}", e)?;
        }
        Ok(())
    }
}

impl<S: Span> std::error::Error for Errors<S> {}

impl<S: Span> AddAssign for Errors<S> {
    fn add_assign(&mut self, rhs: Self) {
        self.0.extend(rhs.0.into_iter());
    }
}

impl RootSchema {
    /// Validate a value against the schema.
    pub fn validate<V: ?Sized + Validate>(&self, value: &V) -> Result<(), Errors<V::Span>> {
        self.schema.validate(value)
    }
}

impl Schema {
    /// Validate a value against the schema.
    pub fn validate<V: ?Sized + Validate>(&self, value: &V) -> Result<(), Errors<V::Span>> {
        let r = SchemaRef::from(self);
        let s = not_bool_schema!(r, &value.span());
        s.validate(value)
    }
}

impl SchemaObject {
    /// Validate a value against the schema.
    pub fn validate<V: ?Sized + Validate>(&self, value: &V) -> Result<(), Errors<V::Span>> {
        let mut errors = validate_subschemas(self, value).err();

        if let Err(e) = value.validate(SchemaValidator::from_ref(SchemaRef::Object(self))) {
            match &mut errors {
                Some(errs) => {
                    *errs += e;
                }
                None => errors = Some(e),
            }
        }

        match errors {
            None => Ok(()),
            Some(e) => Err(e),
        }
    }
}

/// Validate all the allOf anyOf, etc. schemas for a given value.
fn validate_subschemas<V: ?Sized + Validate>(
    schema: &SchemaObject,
    value: &V,
) -> Result<(), Errors<V::Span>> {
    if let Some(sub) = &schema.subschemas {
        let mut errors = SmallVec::new();

        if let Some(all_of) = &sub.all_of {
            for s in all_of {
                if let Err(e) = s.validate(value) {
                    errors.extend(e.0.into_iter());
                }
            }
        }

        if let Some(any_of) = &sub.any_of {
            let mut validated = Vec::with_capacity(any_of.len());
            let mut inner_errors: Vec<Errors<_>> = Vec::with_capacity(any_of.len());
            for s in any_of {
                match s.validate(value) {
                    Ok(_) => match s {
                        Schema::Object(o) => {
                            validated.push(o.metadata.clone());
                        }
                        _ => {
                            validated.push(None);
                        }
                    },
                    Err(e) => {
                        inner_errors.push(e);
                    }
                }
            }
            if validated.is_empty() {
                errors.push(Error::new(
                    schema.metadata.clone(),
                    value.span(),
                    ErrorValue::NoneValid {
                        errors: inner_errors,
                    },
                ));
            } else if validated.len() > 1 {
                errors.push(Error::new(
                    schema.metadata.clone(),
                    value.span(),
                    ErrorValue::MoreThanOneValid { matched: validated },
                ));
            }
        }

        if let Some(one_of) = &sub.one_of {
            let mut validated = Vec::with_capacity(one_of.len());
            let mut inner_errors: Vec<Errors<_>> = Vec::with_capacity(one_of.len());
            for s in one_of {
                match s.validate(value) {
                    Ok(_) => match s {
                        Schema::Object(o) => {
                            validated.push(o.metadata.clone());
                        }
                        _ => {
                            validated.push(None);
                        }
                    },
                    Err(e) => {
                        inner_errors.push(e);
                    }
                }
            }
            if validated.is_empty() {
                errors.push(Error::new(
                    schema.metadata.clone(),
                    value.span(),
                    ErrorValue::NoneValid {
                        errors: inner_errors,
                    },
                ));
            } else if validated.len() > 1 {
                errors.push(Error::new(
                    schema.metadata.clone(),
                    value.span(),
                    ErrorValue::MoreThanOneValid { matched: validated },
                ));
            }
        }

        if let (Some(sub_if), Some(sub_then)) = (&sub.if_schema, &sub.then_schema) {
            if sub_if.validate(value).is_ok() {
                if let Err(e) = sub_then.validate(value) {
                    errors.extend(e.0.into_iter());
                }
            } else if let Some(sub_else) = &sub.else_schema {
                if let Err(e) = sub_else.validate(value) {
                    errors.extend(e.0.into_iter());
                }
            }
        }

        if let Some(not) = &sub.not {
            if not.validate(value).is_ok() {
                errors.push(Error::new(
                    schema.metadata.clone(),
                    value.span(),
                    ErrorValue::ValidNot {
                        matched: match &**not {
                            Schema::Object(o) => o.metadata.clone(),
                            _ => None,
                        },
                    },
                ));
            }
        }

        return if errors.is_empty() {
            Ok(())
        } else {
            Err(Errors(errors))
        };
    }

    Ok(())
}

/// This is technically not needed anymore,
/// but should do no harm to leave it as is.
enum SchemaRef<'s> {
    Bool(bool),
    Object(&'s SchemaObject),
}

impl<'s> From<&'s Schema> for SchemaRef<'s> {
    fn from(s: &'s Schema) -> Self {
        match s {
            Schema::Bool(b) => SchemaRef::Bool(*b),
            Schema::Object(o) => SchemaRef::Object(o),
        }
    }
}

impl<'s> From<&'s RootSchema> for SchemaRef<'s> {
    fn from(s: &'s RootSchema) -> Self {
        SchemaRef::Object(&s.schema)
    }
}

/// A validator that validates a given schema.
///
/// This is not exposed directly because a value must be validated
/// against multiple schemas in some cases, so the `Schema::validate` methods
/// must be used instead that will validate the value against their subschemas.
struct SchemaValidator<'a, S: Span> {
    schema: SchemaRef<'a>,
    span: Option<S>,

    // Array tracking
    arr_item_count: usize,
    // For unique checks
    arr_hashes: HashMap<u64, Option<S>>,
    arr_contains: Option<&'a Schema>,

    // Object tracking
    obj_required: Set<String>,
    obj_prop_count: usize,
    obj_last_key: Option<String>,
}

impl<'a, S: Span> SchemaValidator<'a, S> {
    fn from_ref(schema: SchemaRef<'a>) -> Self {
        Self {
            schema,
            span: None,
            arr_item_count: 0,
            arr_hashes: HashMap::new(),
            arr_contains: None,
            obj_required: Set::new(),
            obj_prop_count: 0,
            obj_last_key: None,
        }
    }
}

impl<'a, S: Span> Validator<S> for SchemaValidator<'a, S> {
    type Error = Errors<S>;

    type ValidateSeq = Self;
    type ValidateMap = Self;

    fn with_span(mut self, span: Option<S>) -> Self {
        self.span = span;
        self
    }

    fn validate_bool(self, v: bool) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Boolean, s, &self.span)?;
        check_enum!(bool, v, s, &self.span)?;

        Ok(())
    }

    fn validate_i8(self, v: i8) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_i16(self, v: i16) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_i32(self, v: i32) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_i64(self, v: i64) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_i128(self, v: i128) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u8(self, v: u8) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u16(self, v: u16) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u32(self, v: u32) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u64(self, v: u64) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u128(self, v: u128) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_f32(self, v: f32) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Number, s, &self.span)?;
        check_enum!(float, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_f64(self, v: f64) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Number, s, &self.span)?;
        check_enum!(float, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_char(self, v: char) -> Result<(), Self::Error> {
        self.validate_str(&v.to_string())
    }

    fn validate_str(self, v: &str) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(String, s, &self.span)?;
        check_enum!(str, v, s, &self.span)?;
        check_string!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_bytes(self, _v: &[u8]) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(String, s, &self.span)?;
        // TODO anything else to check here?
        Ok(())
    }

    fn validate_none(self) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Null, s, &self.span)?;

        Ok(())
    }

    fn validate_some<T>(self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Validate<Span = S>,
    {
        value.validate(self)
    }

    fn validate_unit(self) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Null, s, &self.span)?;

        Ok(())
    }

    fn validate_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        check_type!(Null, s, &self.span)?;

        Ok(())
    }

    fn validate_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), Self::Error> {
        self.validate_str(variant)
    }

    fn validate_seq(mut self, len: Option<usize>) -> Result<Self::ValidateSeq, Self::Error> {
        let s = not_bool_schema!(self, &self.schema, &self.span);

        check_type!(Array, s, &self.span)?;

        if let Some(arr) = &s.array {
            if let Some(s) = &arr.contains {
                self.arr_contains = Some(s);
            }
        }
        if let Some(l) = len {
            self.arr_hashes.reserve(l);
        }
        Ok(self)
    }

    fn validate_map(mut self, _len: Option<usize>) -> Result<Self::ValidateMap, Self::Error> {
        let s = not_bool_schema!(self, &self.schema, &self.span);

        check_type!(Object, s, &self.span)?;

        if let Some(obj) = &s.object {
            self.obj_required = obj.required.clone();
        }

        Ok(self)
    }
}

impl<'a, S: Span> ValidateSeq<S> for SchemaValidator<'a, S> {
    type Error = Errors<S>;

    fn set_span(&mut self, span: Option<S>) {
        self.span = span;
    }

    fn validate_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Validate<Span = S> + Hash,
    {
        let s = not_bool_schema!(&self.schema, &self.span);

        let mut errors = Errors::new();

        self.arr_item_count += 1;
        if let Some(arr) = &s.array {
            if let Some(c) = self.arr_contains {
                if c.validate(value).is_ok() {
                    self.arr_contains = None;
                }
            }

            if let Some(items) = &arr.items {
                match items {
                    SingleOrVec::Single(single_schema) => {
                        if let Err(e) = single_schema.validate(value) {
                            errors.0.extend(e.0.into_iter());
                        }
                    }
                    SingleOrVec::Vec(schemas) => {
                        if let Some(s) = schemas.get(self.arr_item_count - 1) {
                            if let Err(e) = s.validate(value) {
                                errors.0.extend(e.0.into_iter());
                            }
                        } else if let Some(s) = &arr.additional_items {
                            if let Err(e) = s.validate(value) {
                                errors.0.extend(e.0.into_iter());
                            }
                        }
                    }
                }
            }

            if let Some(true) = arr.unique_items {
                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                let h = hasher.finish();

                let existing = self.arr_hashes.insert(h, value.span());

                if let Some(existing_val) = existing {
                    errors.0.push(Error::new(
                        s.metadata.clone(),
                        value.span(),
                        ErrorValue::NotUnique {
                            first: existing_val,
                            duplicate: value.span(),
                        },
                    ));
                }
            }
        }

        if !errors.0.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn end(self) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);
        let mut errors = Errors::new();

        if let Some(c) = self.arr_contains {
            errors.0.push(Error::new(
                s.metadata.clone(),
                self.span.clone(),
                ErrorValue::MustContain {
                    schema: match c {
                        Schema::Bool(_) => None,
                        Schema::Object(o) => o.metadata.clone(),
                    },
                },
            ));
        }

        if let Some(arr) = &s.array {
            if let Some(min) = arr.min_items {
                if self.arr_item_count < min as usize {
                    errors.0.push(Error::new(
                        s.metadata.clone(),
                        self.span.clone(),
                        ErrorValue::NotEnoughItems { min: min as usize },
                    ));
                }
            }

            if let Some(max) = arr.max_items {
                if self.arr_item_count > max as usize {
                    errors.0.push(Error::new(
                        s.metadata.clone(),
                        self.span.clone(),
                        ErrorValue::TooManyItems { max: max as usize },
                    ));
                }
            }
        }

        if !errors.0.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

impl<'a, S: Span> ValidateMap<S> for SchemaValidator<'a, S> {
    type Error = Errors<S>;

    fn set_span(&mut self, span: Option<S>) {
        self.span = span;
    }

    fn validate_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Validate<Span = S> + ToString,
    {
        let s = not_bool_schema!(&self.schema, &self.span);

        let key_string = key.to_string();

        self.obj_prop_count += 1;
        self.obj_required.remove(&key_string);

        self.obj_last_key = Some(key_string);

        if let Some(obj) = &s.object {
            if let Some(name_schema) = &obj.property_names {
                if let Err(e) = name_schema.validate(key) {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn validate_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Validate<Span = S>,
    {
        let s = not_bool_schema!(&self.schema, &self.span);
        let key = self.obj_last_key.take().expect("no key before value");

        if let Some(obj) = &s.object {
            if let Some(prop_schema) = obj.properties.get(&key) {
                match prop_schema.validate(value) {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(e);
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
                            return Err(e);
                        }
                    }
                }
            }

            if let Some(add_prop_schema) = &obj.additional_properties {
                if let Err(e) = add_prop_schema.validate(value) {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn end(self) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);
        let mut errors = Errors::new();

        if let Some(obj) = &s.object {
            if let Some(max) = obj.max_properties {
                if self.obj_prop_count > max as usize {
                    errors.0.push(Error::new(
                        s.metadata.clone(),
                        self.span.clone(),
                        ErrorValue::TooManyProperties { max: max as usize },
                    ))
                }
            }

            if let Some(min) = obj.min_properties {
                if self.obj_prop_count < min as usize {
                    errors.0.push(Error::new(
                        s.metadata.clone(),
                        self.span.clone(),
                        ErrorValue::NotEnoughProperties { min: min as usize },
                    ))
                }
            }
        }

        for p in self.obj_required {
            errors.0.push(Error::new(
                s.metadata.clone(),
                self.span.clone(),
                ErrorValue::RequiredProperty { name: p },
            ))
        }

        if !errors.0.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}
