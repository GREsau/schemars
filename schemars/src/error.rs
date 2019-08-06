use std::fmt;
use std::error::Error;
use crate::schema::Schema;

pub type Result<T = Schema> = std::result::Result<T, MakeSchemaError>;

#[derive(Debug, Clone)]
pub struct MakeSchemaError {
    msg: &'static str,
    schema: Schema
}

impl MakeSchemaError {
    pub fn new(msg: &'static str, schema: Schema) -> MakeSchemaError {
        MakeSchemaError { msg, schema }
    }
}

impl fmt::Display for MakeSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Schema: {:?}", self.msg, self.schema)
    }
}

impl Error for MakeSchemaError {
}
