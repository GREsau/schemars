use syn::{Attribute, Lit::Str, Meta::NameValue, MetaNameValue};

pub fn get_title_and_desc_from_doc(attrs: &[Attribute]) -> (Option<String>, Option<String>) {
    let preserve_formatting = should_preserve_formatting(attrs);

    let doc = match get_doc(attrs, preserve_formatting) {
        None => return (None, None),
        Some(doc) => doc,
    };

    let (title, mut maybe_desc) = if doc.starts_with('#') {
        let mut split = doc.splitn(2, '\n');
        let title = split
            .next()
            .unwrap()
            .trim_start_matches('#')
            .trim()
            .to_owned();
        (none_if_empty(title), split.next().map(ToOwned::to_owned))
    } else {
        (None, Some(doc))
    };

    if !preserve_formatting {
        maybe_desc = maybe_desc.as_ref().map(String::as_str).and_then(merge_description_lines);
    }

    (title, maybe_desc)
}

fn merge_description_lines(doc: &str) -> Option<String> {
    let desc = doc
        .trim()
        .split("\n\n")
        .filter_map(|line| none_if_empty(line.trim().replace('\n', " ")))
        .collect::<Vec<_>>()
        .join("\n\n");
    none_if_empty(desc)
}

fn should_preserve_formatting(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path.is_ident("schemars_preserve_doc_formatting"))
}

fn get_doc(attrs: &[Attribute], preserve_formatting: bool) -> Option<String> {
    let doc = attrs
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

    let doc = if !preserve_formatting {
        doc
            .iter()
            .flat_map(|a| a.split('\n'))
            .map(str::trim)
            .skip_while(|s| *s == "")
            .collect::<Vec<_>>()
            .join("\n")
    } else if !doc.is_empty() && doc.iter().all(|line| line.starts_with(' ')) {
        doc.iter().map(|line| &line[1..]).collect::<Vec<_>>().join("\n")
    } else {
        doc.join("\n")
    };

    none_if_empty(doc)
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
