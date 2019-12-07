use syn::{Attribute, Lit::Str, Meta::NameValue, MetaNameValue};

pub fn get_title_and_desc_from_docs(attrs: &[Attribute]) -> (Option<String>, Option<String>) {
    let docs = match get_docs(attrs) {
        None => return (None, None),
        Some(docs) => docs,
    };

    if docs.starts_with('#') {
        let mut split = docs.splitn(2, '\n');
        let title = split
            .next()
            .unwrap()
            .trim_start_matches('#')
            .trim()
            .to_owned();
        let maybe_desc = split.next().and_then(get_description);
        (none_if_empty(title), maybe_desc)
    } else {
        (None, get_description(&docs))
    }
}

fn get_description(docs: &str) -> Option<String> {
    let desc = docs
        .trim()
        .split("\n\n")
        .filter_map(|line| none_if_empty(line.trim().replace('\n', " ")))
        .collect::<Vec<_>>()
        .join("\n\n");
    none_if_empty(desc)
}

fn get_docs(attrs: &[Attribute]) -> Option<String> {
    let docs = attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path.is_ident("doc") {
                return None;
            }

            let meta = attr.parse_meta().ok()?;
            if let NameValue(MetaNameValue { lit: Str(s), .. }) = meta {
                return Some(s.value());
            }

            None
        })
        .collect::<Vec<_>>()
        .iter()
        .flat_map(|a| a.split('\n'))
        .map(str::trim)
        .skip_while(|s| *s == "")
        .collect::<Vec<_>>()
        .join("\n");
    none_if_empty(docs)
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
