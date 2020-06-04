use super::{Validate, ValidateMap, ValidateSeq, Validator};
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

/// An validation error with an optional span.
#[derive(Debug, Clone, PartialEq)]
pub struct Error<S: core::fmt::Debug + Clone> {
    pub meta: Option<Box<Metadata>>,

    pub span: Option<S>,

    /// The actual error details.
    pub value: ErrorValue<S>,
}

impl<S: core::fmt::Debug + Clone> Error<S> {
    fn new(meta: Option<Box<Metadata>>, span: Option<S>, value: ErrorValue<S>) -> Self {
        Self { meta, span, value }
    }
}

impl<S: core::fmt::Debug + Clone> core::fmt::Display for Error<S> {
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
pub enum ErrorValue<S: core::fmt::Debug + Clone> {
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

impl<S: core::fmt::Debug + Clone> core::fmt::Display for ErrorValue<S> {
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

type ErrorsInner<S> = SmallVec<[Error<S>; 10]>;

// TODO vec impls
/// This type is returned from validation.
#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct Errors<S: core::fmt::Debug + Clone>(ErrorsInner<S>);

impl<S: core::fmt::Debug + Clone> Errors<S> {
    fn new() -> Self {
        Errors(SmallVec::new())
    }
}

impl<S: core::fmt::Debug + Clone> core::fmt::Display for Errors<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.0 {
            writeln!(f, "{}", e)?;
        }
        Ok(())
    }
}

impl<S: core::fmt::Debug + Clone> std::error::Error for Errors<S> {}

impl<S: core::fmt::Debug + Clone> AddAssign for Errors<S> {
    fn add_assign(&mut self, rhs: Self) {
        self.0.extend(rhs.0.into_iter());
    }
}

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

pub struct SchemaValidator<'a, S: core::fmt::Debug + Clone> {
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

impl<'a, S: core::fmt::Debug + Clone> SchemaValidator<'a, S> {
    pub fn new(schema: &'a Schema, span: Option<S>) -> Self {
        Self::from_ref(schema.into(), span)
    }

    pub fn new_root(root_schema: &'a RootSchema, span: Option<S>) -> Self {
        Self::from_ref(root_schema.into(), span)
    }

    fn from_ref(schema: SchemaRef<'a>, span: Option<S>) -> Self {
        Self {
            span,
            schema,
            arr_item_count: 0,
            arr_hashes: HashMap::new(),
            arr_contains: None,
            obj_required: Set::new(),
            obj_prop_count: 0,
            obj_last_key: None,
        }
    }
}

macro_rules! not_bool_schema {
    ($schema:expr, $span:expr) => {
        match &$schema {
            SchemaRef::Bool(allow_all) => {
                if *allow_all {
                    return Ok(());
                } else {
                    let mut errors = SmallVec::new();
                    errors.push(Error::new(None, (*$span).clone(), ErrorValue::NotAllowed));
                    return Err(Errors(errors));
                }
            }
            SchemaRef::Object(o) => o,
        }
    };

    // With a non-unit return type.
    ($ret_val:expr, $schema:expr, $span:expr) => {
        match &$schema {
            SchemaRef::Bool(allow_all) => {
                if *allow_all {
                    return Ok($ret_val);
                } else {
                    let mut errors = SmallVec::new();
                    errors.push(Error::new(None, (*$span).clone(), ErrorValue::NotAllowed));
                    return Err(Errors(errors));
                }
            }
            SchemaRef::Object(o) => o,
        }
    };
}

macro_rules! check_type {
    ($expected_type:ident, $schema:expr, $span:expr) => {
        match &$schema.instance_type {
            Some(s) => match s {
                SingleOrVec::Single(single) => match &**single {
                    InstanceType::$expected_type => Ok(()),
                    _ => {
                        let mut errors = SmallVec::new();
                        errors.push(Error::new(
                            $schema.metadata.clone(),
                            $span.clone(),
                            ErrorValue::InvalidType {
                                expected: s.clone(),
                            },
                        ));
                        Err(Errors(errors))
                    }
                },
                SingleOrVec::Vec(vec) => {
                    if vec.iter().any(|i| *i == InstanceType::$expected_type) {
                        Ok(())
                    } else {
                        let mut errors = SmallVec::new();
                        errors.push(Error::new(
                            $schema.metadata.clone(),
                            $span.clone(),
                            ErrorValue::InvalidType {
                                expected: s.clone(),
                            },
                        ));
                        Err(Errors(errors))
                    }
                }
            },
            None => {
                let mut errors = SmallVec::new();
                errors.push(Error::new(
                    $schema.metadata.clone(),
                    $span.clone(),
                    ErrorValue::InvalidType {
                        expected: SingleOrVec::Single(Box::new(InstanceType::Object)),
                    },
                ));
                Err(Errors(errors))
            }
        }
    };
    (Object, $schema:expr, $span:expr) => {
        match &$schema.instance_type {
            Some(s) => match s {
                SingleOrVec::Single(single) => match &**single {
                    InstanceType::Object => Ok(()),
                    _ => {
                        let mut errors = SmallVec::new();
                        errors.push(Error::new(
                            $schema.metadata.clone(),
                            $span.clone(),
                            ErrorValue::InvalidType {
                                expected: s.clone(),
                            },
                        ));
                        Err(Errors(errors))
                    }
                },
                SingleOrVec::Vec(vec) => {
                    if vec.iter().any(|i| *i == InstanceType::Object) {
                        Ok(())
                    } else {
                        let mut errors = SmallVec::new();
                        errors.push(Error::new(
                            $schema.metadata.clone(),
                            $span.clone(),
                            ErrorValue::InvalidType {
                                expected: s.clone(),
                            },
                        ));
                        Err(Errors(errors))
                    }
                }
            },
            None => Ok(()),
        }
    };
}

macro_rules! check_enum {
    (bool, $value:expr, $schema:expr, $span:expr) => {
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
                let mut errors = SmallVec::new();
                errors.push(Error::new(
                    $schema.metadata.clone(),
                    $span.clone(),
                    ErrorValue::InvalidEnumValue {
                        expected: enum_vals.clone(),
                    },
                ));
                Err(Errors(errors))
            }
        } else {
            Ok(())
        }
    };
    (int, $value:expr, $schema:expr, $span:expr) => {
        if let Some(enum_vals) = &$schema.enum_values {
            let mut errors = SmallVec::new();

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
                errors.push(Error::new(
                    $schema.metadata.clone(),
                    $span.clone(),
                    ErrorValue::InvalidEnumValue {
                        expected: enum_vals.clone(),
                    },
                ));
                Err(Errors(errors))
            }
        } else {
            Ok(())
        }
    };
    (float, $value:expr, $schema:expr, $span:expr) => {
        if let Some(enum_vals) = &$schema.enum_values {
            let mut errors = SmallVec::new();

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
                errors.push(Error::new(
                    $schema.metadata.clone(),
                    $span.clone(),
                    ErrorValue::InvalidEnumValue {
                        expected: enum_vals.clone(),
                    },
                ));
                Err(Errors(errors))
            }
        } else {
            Ok(())
        }
    };
    (str, $value:expr, $schema:expr, $span:expr) => {
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
                let mut errors = SmallVec::new();
                errors.push(Error::new(
                    $schema.metadata.clone(),
                    $span.clone(),
                    ErrorValue::InvalidEnumValue {
                        expected: enum_vals.clone(),
                    },
                ));
                Err(Errors(errors))
            }
        } else {
            Ok(())
        }
    };
}

// TODO float conversions and equality
macro_rules! check_number {
    ($value:expr, $schema:expr, $span:expr) => {
        if let Some(n) = &$schema.number {
            let mut errors = SmallVec::new();

            let mut number_err = false;

            if let Some(m) = n.multiple_of {
                if m != 0f64 && $value as f64 % m != 0f64 {
                    errors.push(Error::new(
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::NotMultipleOf { multiple_of: m },
                    ));
                    number_err = true;
                }
            }

            if let Some(min) = n.minimum {
                if ($value as f64) < min {
                    errors.push(Error::new(
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::LessThanExpected {
                            min,
                            exclusive: false,
                        },
                    ));
                    number_err = true;
                }
            }

            if let Some(min) = n.exclusive_minimum {
                if ($value as f64) <= min {
                    errors.push(Error::new(
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::LessThanExpected {
                            min,
                            exclusive: true,
                        },
                    ));
                    number_err = true;
                }
            }

            if let Some(max) = n.maximum {
                if ($value as f64) > max {
                    errors.push(Error::new(
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::MoreThanExpected {
                            max,
                            exclusive: false,
                        },
                    ));
                    number_err = true;
                }
            }

            if let Some(max) = n.exclusive_maximum {
                if ($value as f64) >= max {
                    errors.push(Error::new(
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::MoreThanExpected {
                            max,
                            exclusive: true,
                        },
                    ));
                    number_err = true;
                }
            }
            if !number_err {
                Ok(())
            } else {
                Err(Errors(errors))
            }
        } else {
            Ok(())
        };
    };
}

// TODO format
macro_rules! check_string {
    ($value:expr, $schema:expr, $span:expr) => {
        if let Some(s) = &$schema.string {
            let mut errors = SmallVec::new();

            let mut string_err = false;

            if let Some(p) = &s.pattern {
                // TODO we assume that the regex in the schema is valid
                let re = regex::Regex::new(&*p).unwrap();
                if !re.is_match($value) {
                    errors.push(Error::new(
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::NoPatternMatch { pattern: p.clone() },
                    ));
                    string_err = true;
                }

                if let Some(max_length) = s.max_length {
                    if $value.chars().count() > max_length as usize {
                        errors.push(Error::new(
                            $schema.metadata.clone(),
                            $span.clone(),
                            ErrorValue::TooLong { max_length },
                        ));
                        string_err = true;
                    }
                }

                if let Some(min_length) = s.min_length {
                    if $value.chars().count() < min_length as usize {
                        errors.push(Error::new(
                            $schema.metadata.clone(),
                            $span.clone(),
                            ErrorValue::TooShort { min_length },
                        ));
                        string_err = true;
                    }
                }
            }

            if !string_err {
                Ok(())
            } else {
                Err(Errors(errors))
            }
        } else {
            Ok(())
        }
    };
}

macro_rules! check_subschemas {
    ($func:ident, $schema:expr, $span:expr) => {
        // check_subschemas!($func, $schema, $span,)
    };
    ($func:ident, $schema:expr, $span:expr, $($values:expr),*) => {
        if let Some(sub) = &$schema.subschemas {
            let mut errors = SmallVec::new();

            if let Some(all_of) = &sub.all_of {
                for s in all_of {
                    if let Err(e) = (SchemaValidator::new(s, $span.clone())).$func($($values),*) {
                        errors.extend(e.0.into_iter());
                    }
                }
            }

            if let Some(any_of) = &sub.any_of {
                let mut validated = Vec::with_capacity(any_of.len());
                let mut inner_errors: Vec<Errors<_>> = Vec::with_capacity(any_of.len());
                for s in any_of {
                    match (SchemaValidator::new(s, $span.clone())).$func($($values),*) {
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
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::NoneValid {
                            errors: inner_errors,
                        },
                    ));
                } else if validated.len() > 1 {
                    errors.push(Error::new(
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::MoreThanOneValid { matched: validated },
                    ));
                }
            }

            if let Some(one_of) = &sub.one_of {
                let mut validated = Vec::with_capacity(one_of.len());
                let mut inner_errors: Vec<Errors<_>> = Vec::with_capacity(one_of.len());
                for s in one_of {
                    match (SchemaValidator::new(s, $span.clone())).$func($($values),*) {
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
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::NoneValid {
                            errors: inner_errors,
                        },
                    ));
                } else if validated.len() > 1 {
                    errors.push(Error::new(
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::MoreThanOneValid { matched: validated },
                    ));
                }
            }

            if let (Some(sub_if), Some(sub_then)) = (&sub.if_schema, &sub.then_schema) {
                if let Ok(_) = (SchemaValidator::new(sub_if, $span.clone())).$func($($values),*) {
                    if let Err(e) = (SchemaValidator::new(sub_then, $span.clone())).$func($($values),*) {
                        errors.extend(e.0.into_iter());
                    }
                } else {
                    if let Some(sub_else) = &sub.else_schema {
                        if let Err(e) =
                            (SchemaValidator::new(sub_else, $span.clone())).$func($($values),*)
                        {
                            errors.extend(e.0.into_iter());
                        }
                    }
                }
            }

            if let Some(not) = &sub.not {
                if let Ok(_) = (SchemaValidator::new(not, $span.clone())).$func($($values),*) {
                    errors.push(Error::new(
                        $schema.metadata.clone(),
                        $span.clone(),
                        ErrorValue::ValidNot { matched: match &**not {
                            Schema::Object(o) => {
                                    o.metadata.clone()
                                },
                                _ => None
                            }
                        },
                    ));
                }
            }

            if errors.len() == 1 {
                Ok(())
            } else {
                Err(Errors(errors))
            }
        } else {
            Ok(())
        }
    };
}

impl<'a, S: core::fmt::Debug + Clone> Validator<S> for SchemaValidator<'a, S> {
    type Error = Errors<S>;

    type ValidateSeq = Self;
    type ValidateMap = Self;

    fn with_span(mut self, span: Option<S>) -> Self {
        self.span = span;
        self
    }

    fn validate_bool(self, v: bool) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_bool, s, &self.span, v)?;
        check_type!(Boolean, s, &self.span)?;
        check_enum!(bool, v, s, &self.span)?;

        Ok(())
    }

    fn validate_i8(self, v: i8) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_i8, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_i16(self, v: i16) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_i16, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_i32(self, v: i32) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_i32, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_i64(self, v: i64) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_i64, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_i128(self, v: i128) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_i128, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u8(self, v: u8) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_u8, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u16(self, v: u16) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_u16, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u32(self, v: u32) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_u32, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u64(self, v: u64) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_u64, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_u128(self, v: u128) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_u128, s, &self.span, v)?;
        check_type!(Integer, s, &self.span)?;
        check_enum!(int, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_f32(self, v: f32) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_f32, s, &self.span, v)?;
        check_type!(Number, s, &self.span)?;
        check_enum!(float, v, s, &self.span)?;
        check_number!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_f64(self, v: f64) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_f64, s, &self.span, v)?;
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

        // check_subschemas!(validate_str, s, &self.span, v)?;
        check_type!(String, s, &self.span)?;
        check_enum!(str, v, s, &self.span)?;
        check_string!(v, s, &self.span)?;

        Ok(())
    }

    fn validate_bytes(self, v: &[u8]) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_bytes, s, &self.span, v)?;
        check_type!(String, s, &self.span)?;
        // TODO anything else to check here?
        Ok(())
    }

    fn validate_none(self) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_none, s, &self.span,)?;
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

        // check_subschemas!(validate_unit, s, &self.span,)?;
        check_type!(Null, s, &self.span)?;

        Ok(())
    }

    fn validate_unit_struct(self, name: &'static str) -> Result<(), Self::Error> {
        let s = not_bool_schema!(&self.schema, &self.span);

        // check_subschemas!(validate_unit_struct, s, &self.span, name)?;
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

        // check_subschemas!(validate_seq, s, &self.span, len)?;

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

    fn validate_map(mut self, len: Option<usize>) -> Result<Self::ValidateMap, Self::Error> {
        let s = not_bool_schema!(self, &self.schema, &self.span);

        // check_subschemas!(validate_map, s, &self.span, len)?;

        check_type!(Object, s, &self.span)?;

        if let Some(obj) = &s.object {
            self.obj_required = obj.required.clone();
        }

        Ok(self)
    }
}

impl<'a, S: core::fmt::Debug + Clone> ValidateSeq<S> for SchemaValidator<'a, S> {
    type Error = Errors<S>;

    fn validate_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Validate<Span = S> + Hash,
    {
        let s = not_bool_schema!(&self.schema, &self.span);

        let mut errors = Errors::new();

        self.arr_item_count += 1;
        if let Some(arr) = &s.array {
            if let Some(c) = self.arr_contains {
                if let Ok(_) = value.validate(SchemaValidator::new(c, self.span.clone())) {
                    self.arr_contains = None;
                }
            }

            if let Some(items) = &arr.items {
                match items {
                    SingleOrVec::Single(single_schema) => {
                        if let Err(e) =
                            value.validate(SchemaValidator::new(single_schema, value.span()))
                        {
                            errors.0.extend(e.0.into_iter());
                        }
                    }
                    SingleOrVec::Vec(schemas) => {
                        if let Some(s) = schemas.get(self.arr_item_count - 1) {
                            if let Err(e) =
                                value.validate(SchemaValidator::new(s, value.span()))
                            {
                                errors.0.extend(e.0.into_iter());
                            }
                        } else if let Some(s) = &arr.additional_items {
                            if let Err(e) =
                                value.validate(SchemaValidator::new(s, value.span()))
                            {
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

impl<'a, S: core::fmt::Debug + Clone> ValidateMap<S> for SchemaValidator<'a, S> {
    type Error = Errors<S>;

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
                if let Err(e) = key.validate(SchemaValidator::new(name_schema, key.span())) {
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
                match value.validate(SchemaValidator::new(prop_schema, value.span())) {
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
                    match value.validate(SchemaValidator::new(v, value.span())) {
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
                if let Err(e) = value.validate(SchemaValidator::new(add_prop_schema, value.span()))
                {
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

// /// Returned if a map key is not string, as json
// /// only supports string keys.
// #[derive(Debug, Copy, Clone, Eq, PartialEq)]
// pub struct KeyNotStringError;

// impl core::fmt::Debug for KeyNotStringError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str("json keys must be strings")
//     }
// }

// impl std::error::Error for KeyNotStringError {}

// impl ser::Error for KeyNotStringError {
//     fn custom<T>(msg: T) -> Self
//     where
//         T: std::fmt::Display,
//     {
//         unreachable!()
//     }
// }

// /// A serializer that only allows strings.
// ///
// /// It converts integers to strings just like serde_json does.
// struct KeySerializer;

// impl Validator for KeySerializer {
//     type Ok = String;
//     type Error = KeyNotStringError;
//     type SerializeSeq = ser::Impossible<String, KeyNotStringError>;
//     type SerializeTuple = ser::Impossible<String, KeyNotStringError>;
//     type SerializeTupleStruct = ser::Impossible<String, KeyNotStringError>;
//     type SerializeTupleVariant = ser::Impossible<String, KeyNotStringError>;
//     type SerializeMap = ser::Impossible<String, KeyNotStringError>;
//     type SerializeStruct = ser::Impossible<String, KeyNotStringError>;
//     type SerializeStructVariant = ser::Impossible<String, KeyNotStringError>;

//     fn validate_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_char(self, v: char) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
//         Ok(v.to_string())
//     }
//     fn validate_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_none(self) -> Result<Self::Ok, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
//     where
//         T: Serialize,
//     {
//         Err(KeyNotStringError)
//     }
//     fn validate_unit(self) -> Result<Self::Ok, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_unit_variant(
//         self,
//         name: &'static str,
//         variant_index: u32,
//         variant: &'static str,
//     ) -> Result<Self::Ok, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_newtype_struct<T: ?Sized>(
//         self,
//         name: &'static str,
//         value: &T,
//     ) -> Result<Self::Ok, Self::Error>
//     where
//         T: Serialize,
//     {
//         Err(KeyNotStringError)
//     }
//     fn validate_newtype_variant<T: ?Sized>(
//         self,
//         name: &'static str,
//         variant_index: u32,
//         variant: &'static str,
//         value: &T,
//     ) -> Result<Self::Ok, Self::Error>
//     where
//         T: Serialize,
//     {
//         Err(KeyNotStringError)
//     }
//     fn validate_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_tuple_struct(
//         self,
//         name: &'static str,
//         len: usize,
//     ) -> Result<Self::SerializeTupleStruct, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_tuple_variant(
//         self,
//         name: &'static str,
//         variant_index: u32,
//         variant: &'static str,
//         len: usize,
//     ) -> Result<Self::SerializeTupleVariant, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_struct(
//         self,
//         name: &'static str,
//         len: usize,
//     ) -> Result<Self::SerializeStruct, Self::Error> {
//         Err(KeyNotStringError)
//     }
//     fn validate_struct_variant(
//         self,
//         name: &'static str,
//         variant_index: u32,
//         variant: &'static str,
//         len: usize,
//     ) -> Result<Self::SerializeStructVariant, Self::Error> {
//         Err(KeyNotStringError)
//     }
// }
