use alloc::string::String;
use alloc::vec::Vec;

forward_impl!(rocket05::fs::TempFile<'_> => Vec<u8>);
forward_impl!(rocket05::fs::NamedFile => Vec<u8>);

forward_impl!(rocket05::http::RawStr => String);
forward_impl!(rocket05::http::RawStrBuf => String);
