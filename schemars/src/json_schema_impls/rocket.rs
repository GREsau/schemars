use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

forward_impl!(rocket::fs::TempFile<'_> => Vec<u8>);
forward_impl!(rocket::fs::NamedFile => Vec<u8>);

forward_impl!(rocket::http::RawStr => String);
forward_impl!(rocket::http::RawStrBuf => String);
