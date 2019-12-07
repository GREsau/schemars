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
        let maybe_desc = split.next().map(|s| s.trim().to_owned());
        (none_if_empty(title), maybe_desc)
    } else {
        (None, none_if_empty(docs))
    }
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn get_docs(attrs: &[Attribute]) -> Option<String> {
    let doc_attrs = attrs
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
        .collect::<Vec<_>>();

    if doc_attrs.is_empty() {
        return None;
    }

    let mut docs = doc_attrs
        .iter()
        .flat_map(|a| a.split('\n'))
        .map(str::trim)
        .skip_while(|s| *s == "")
        .collect::<Vec<_>>()
        .join("\n");

    docs.truncate(docs.trim_end().len());
    Some(docs)
}
