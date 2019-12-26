---
layout: default
title: Derive for Remote Crate
parent: Examples
nav_order: 5
summary: Deriving JsonSchema implementations for a type in somebody else's crate.
---

# Deriving JsonSchema for a Type in a Different Crate

Rust's [orphan rule](https://doc.rust-lang.org/book/traits.html#rules-for-implementing-traits) requires that either the trait or the type for which you are implementing the trait must be defined in the same crate as the impl, so it is not possible to implement `JsonSchema` for a type in a different crate directly.

To work around this, Schemars provides a way of deriving `JsonSchema` implementations for types in other people's crates. The only catch is that you have to provide a definition of the type for Schemars's derive to process.

This is the same way that Serde allows remote deriving, which is why this page reads so similarly to [Serde's documentation](https://serde.rs/remote-derive.html)!

{% include example.md name="remote_derive" %}
