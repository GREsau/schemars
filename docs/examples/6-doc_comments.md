---
layout: default
title: Doc Comments
parent: Examples
nav_order: 6
summary: Giving schemas a custom title and/or description using doc comments.
---

# Setting a Custom Title and/or Description Using Doc Comments

If a struct, variant or field has any [doc comments](https://doc.rust-lang.org/stable/rust-by-example/meta/doc.html#doc-comments) (or [`doc` attributes](https://doc.rust-lang.org/rustdoc/the-doc-attribute.html)), then these will be used as the generated schema's `description`. If the first line is an ATX-style markdown heading (i.e. it begins with a # character), then it will be used as the schema's `title`, and the remaining lines will be the `description`.

{% include example.md name="doc_comments" %}
