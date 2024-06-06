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
        let maybe_desc = split.next().and_then(merge_description_lines);
        (none_if_empty(title), maybe_desc)
    } else {
        (None, merge_description_lines(&doc))
    }
}

fn merge_description_lines(doc: &str) -> Option<String> {
    let desc = doc
        .trim()
        .split("\n\n")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let desc = desc
        .iter()
        .map(|paragrah| merge_without_codeblock(paragrah))
        .filter_map(|line| none_if_empty(line.to_string()))
        .collect::<Vec<_>>()
        .join("\n\n");
    none_if_empty(desc)
}

fn get_doc(attrs: &[Attribute]) -> Option<String> {
    let attrs = attrs
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
    let lines = attrs
        .iter()
        .flat_map(|a| a.split('\n'))
        .skip_while(|s| s.is_empty())
        .map(|l| l.to_string())
        .collect::<Vec<_>>();

    let mut res = strip_without_codeblock(&lines, |l| l.trim().to_string());
    // Added for backward-compatibility, but perhaps we shouldn't do this
    // https://github.com/rust-lang/rust/issues/32088
    if res.iter().all(|l| l.trim().starts_with('*')) {
        res = res.iter().map(|l| l[1..].to_string()).collect::<Vec<_>>();
        res = strip_without_codeblock(&res, |l| l.trim().to_string());
    };

    none_if_empty(res.join("\n"))
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn strip_without_codeblock(lines: &Vec<String>, func: fn(&str) -> String) -> Vec<String> {
    let mut res = vec![];
    let mut in_codeblock = false;
    for line in lines {
        if line.trim().starts_with("```") {
            in_codeblock = !in_codeblock;
        }
        let l = if in_codeblock {
            line.to_string()
        } else {
            func(line)
        };
        res.push(l);
    }
    while let Some("") = res.first().map(|s| s.as_str()) {
        res.remove(0);
    }
    while let Some("") = res.last().map(|s| s.as_str()) {
        res.pop();
    }
    res
}

fn merge_without_codeblock(content: &str) -> String {
    let lines = content.lines();
    let mut res = String::new();
    let mut in_codeblock = false;
    for line in lines {
        let flag = line.trim().starts_with("```");
        if flag {
            in_codeblock = !in_codeblock;
        }
        // other possible Markdown prefix characters
        let maybe_markdown = ["#", "-", ">", "|", "*", "["]
            .iter()
            .any(|p| line.trim().starts_with(p))
            || line.trim().chars().next().map(char::is_numeric) == Some(true);
        let prefix = if in_codeblock || flag || maybe_markdown {
            "\n"
        } else {
            " "
        };
        res += &(format!("{}{}", prefix, line));
    }
    res.trim().to_string()
}
