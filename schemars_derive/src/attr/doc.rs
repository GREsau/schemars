use syn::{Attribute, Lit::Str, Meta::NameValue, MetaNameValue};

pub fn get_title_and_desc_from_doc(attrs: &[Attribute]) -> (Option<String>, Option<String>) {
    let mut lines = get_lines(attrs);

    let title = lines
        .get(0)
        .as_ref()
        .filter(|line| line.starts_with('#'))
        .map(|line| line.trim_start_matches('#').trim().to_owned())
        .and_then(none_if_empty);

    if title.is_some() {
        lines.remove(0);
        pop_front_empty_lines(&mut lines);
    }

    while let Some(true) = lines.last().map(|x| x.is_empty()) {
        lines.pop();
    }

    (title, none_if_empty(lines.join("\n")))
}

fn get_lines(attrs: &[Attribute]) -> Vec<String> {
    let mut lines = vec![];
    let mut all_starts_with_asterisk = true;

    for attr in attrs.iter() {
        if !attr.path.is_ident("doc") {
            continue;
        }

        if let Ok(NameValue(MetaNameValue { lit: Str(s), .. })) = attr.parse_meta() {
            let value = s.value();

            for line in value.split('\n') {
                let line = line.trim();

                if !line.is_empty() {
                    all_starts_with_asterisk = line.starts_with('*');
                }

                lines.push(line.to_owned());
            }
        }
    }

    // Added for backward-compatibility, but perhaps we shouldn't do this
    // https://github.com/rust-lang/rust/issues/32088
    if all_starts_with_asterisk {
        for line in lines.iter_mut() {
            *line = line.trim_start_matches('*').trim().to_owned();
        }
    }
    pop_front_empty_lines(&mut lines);

    lines
}

fn pop_front_empty_lines(lines: &mut Vec<String>) {
    while let Some(true) = lines.first().map(|x| x.is_empty()) {
        lines.remove(0);
    }
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
