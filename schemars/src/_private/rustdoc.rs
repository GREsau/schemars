#![allow(clippy::all)]
use crate::_alloc_prelude::*;

pub fn get_title_and_description(doc: &str) -> (&str, String) {
    let doc_bytes = trim_ascii(doc.as_bytes());

    if !doc_bytes.is_empty() && doc_bytes[0] == b'#' {
        let title_end_index = match strchr(doc_bytes, b'\n') {
            Some(i) => i,
            None => doc_bytes.len(),
        };

        let title = to_utf8(trim_ascii(trim_start(
            subslice(doc_bytes, 0, title_end_index),
            b'#',
        )));
        let description = merge_description_lines(to_utf8(trim_ascii(subslice(
            doc_bytes,
            title_end_index,
            doc_bytes.len(),
        ))));

        (title, description)
    } else {
        ("", merge_description_lines(to_utf8(doc_bytes)))
    }
}

const fn strchr(bytes: &[u8], chr: u8) -> Option<usize> {
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == chr {
            return Some(i);
        }
        i += 1;
    }
    None
}

const fn subslice(mut bytes: &[u8], mut start: usize, end: usize) -> &[u8] {
    let mut trim_end_count = bytes.len() - end;
    if trim_end_count > 0 {
        while let [rest @ .., _last] = bytes {
            bytes = rest;

            trim_end_count -= 1;
            if trim_end_count == 0 {
                break;
            }
        }
    }

    if start > 0 {
        while let [_first, rest @ ..] = bytes {
            bytes = rest;

            start -= 1;
            if start == 0 {
                break;
            }
        }
    }

    bytes
}

const fn to_utf8(bytes: &[u8]) -> &str {
    match core::str::from_utf8(bytes) {
        Ok(x) => x,
        Err(_) => panic!("Invalid UTF-8"),
    }
}

fn merge_description_lines(doc: &str) -> String {
    doc.trim()
        .split("\n\n")
        .map(|line| line.trim().replace('\n', " "))
        .collect::<Vec<_>>()
        .join("\n\n")
}

const fn trim_start(mut bytes: &[u8], chr: u8) -> &[u8] {
    while let [first, rest @ ..] = bytes {
        if *first == chr {
            bytes = rest;
        } else {
            break;
        }
    }
    bytes
}

const fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = bytes {
        if first.is_ascii_whitespace() {
            bytes = rest;
        } else {
            break;
        }
    }

    while let [rest @ .., last] = bytes {
        if last.is_ascii_whitespace() {
            bytes = rest;
        } else {
            break;
        }
    }

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_title_and_description() {
        let doc = r"
Title

Line1
Line2

Line3
Line4
";
        let (title, description) = get_title_and_description(doc);
        pretty_assertions::assert_eq!(title, "");
        pretty_assertions::assert_eq!(description, "Title\n\nLine1 Line2\n\nLine3 Line4");
    }
}
