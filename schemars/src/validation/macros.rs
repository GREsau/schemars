//! This module contains macros for schema validation, as it is a very
//! boilerplate-y procedure.
//! 
//! Some of the macros below are repeated in every method for Validator.

macro_rules! not_bool_schema {
    ($schema:expr, $span:expr) => {
        not_bool_schema!((), $schema, $span)
    };
    ($ret_val:expr, $schema:expr, $span:expr) => {
        match &$schema {
            SchemaRef::Bool(allow_all) => {
                if *allow_all {
                    return Ok($ret_val);
                } else {
                    let mut errors = SmallVec::new();
                    errors.push(Error::new(None, ($span).clone(), ErrorValue::NotAllowed));
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
                    if (v - $value as f64) < core::f64::EPSILON {
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
