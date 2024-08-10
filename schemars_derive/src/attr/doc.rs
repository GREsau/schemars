use syn::Attribute;

pub fn get_title_and_desc_from_doc(attrs: &[Attribute]) -> (Option<String>, Option<String>) {
    let doc = match get_doc(attrs) {
        None => return (None, None),
        Some(doc) => doc,
    };

    if doc.starts_with('#') {
        let mut split = doc.splitn(2, '\n');
        let title = split
            .next()
            .unwrap()
            .trim_start_matches('#')
            .trim()
            .to_owned();
        let maybe_desc = split.next().map(|s| s.trim().to_owned());
        (none_if_empty(title), maybe_desc)
    } else {
        (None, Some(doc))
    }
}

fn get_doc(attrs: &[Attribute]) -> Option<String> {
    let lines = attrs
        .iter()
        .filter_map(|attr| {
            if !attr.path().is_ident("doc") {
                return None;
            }

            let meta = attr.meta.require_name_value().ok()?;
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) = &meta.value
            {
                return Some(lit_str.value());
            }

            None
        })
        .collect::<Vec<_>>();

    none_if_empty(lines.join("\n").trim().to_owned())
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
