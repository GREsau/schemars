# Changelog

## [1.1.0] - 2025-11-05

### Added

- Public functions that have no side-effects are now marked with [`#[must_use]`](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute) so that they report a lint warning when the returned value is unused, as this likely indicates a mistake.

### Fixed

- Improve accuracy of schemas for flattened enums, in particular: unit variants of externally-tagged enums, and enums wrapped in `Option<>`. (https://github.com/GREsau/schemars/issues/464 / https://github.com/GREsau/schemars/pull/483)

## [1.0.5] - 2025-11-02

### Fixed

- Fix `schema.pointer_mut()` to resolve URI fragment identifiers like `#/$defs/foo`, matching current behaviour of `schema.pointer()` (https://github.com/GREsau/schemars/issues/478 / https://github.com/GREsau/schemars/pull/479)

## [1.0.4] - 2025-07-06

### Fixed

- Fix `JsonSchema` impl on [atomic](https://doc.rust-lang.org/std/sync/atomic/) types being ignored on non-nightly compilers due to a buggy `cfg` check (https://github.com/GREsau/schemars/issues/453)
- Fix compatibility with minimal dependency versions, e.g. old(-ish) versions of `syn` (https://github.com/GREsau/schemars/issues/450)
- Fix derive for empty tuple variants (https://github.com/GREsau/schemars/issues/455)

## [1.0.3] - 2025-06-28

### Fixed

- Fix compile error when a doc comment is set on both a `transparent` (or newtype) struct and its field (https://github.com/GREsau/schemars/issues/446)
- Fix `json_schema!()` macro compatibility when used from pre-2021 rust editions (https://github.com/GREsau/schemars/pull/447)

## [1.0.2] - 2025-06-26

### Fixed

- Fix schema properties being incorrectly reordered during serialization (https://github.com/GREsau/schemars/issues/444)

## [1.0.1] - 2025-06-24

### Fixed

- Deriving `JsonSchema` with `no_std` broken due to `std::borrow::ToOwned` trait not being in scope (https://github.com/GREsau/schemars/issues/441)

## [1.0.0] - 2025-06-23

This is a major release with many additions, fixes and changes since 0.8 (but not many since 0.9). While the basic usage (deriving `JsonSchema` and using `schema_for!()` or `SchemaGenerator`) is mostly unchanged, you may wish to consult the [migration guide](https://graham.cool/schemars/migrating/) which covers some of the most significant changes.

Changes since 1.0.0-rc.2:

### Added

- `#[schemars(bound = ...)]` attributes are now used from fields as well as containers
- The [`Schema::pointer(...)`](https://docs.rs/schemars/1.0.0/schemars/struct.Schema.html#method.pointer) method now works when given a JSON pointer in URI Fragment representation with a leading `#` character. In particular, this means that you can now lookup a schema from a `$ref` value using that method.

### Fixed

- Schema names that contain special characters are now correctly encoded when used inside a `$ref` value (https://github.com/GREsau/schemars/pull/436)
- Optimise type param usage in `SchemaGenerator::subschema_for`, reducing LLVM line count and improving compile times (https://github.com/GREsau/schemars/pull/439)

## [1.0.0-rc.2] - 2025-06-19

### Added

- Serde attributes that you want schemars to ignore can now be "unset" by including them in a schemars attribute with a ! prefix, e.g. `#[schemars(!from)]` (https://github.com/GREsau/schemars/issues/433 / https://github.com/GREsau/schemars/pull/434)

### Removed

- ⚠️ Deprecated items have been removed:
  - `SchemaSettings::option_nullable` and `SchemaSettings::option_add_null_type` fields
  - `gen` module

## [1.0.0-rc.1] - 2025-06-16

### Added

- Impl `JsonSchema` for `chrono::TimeDelta` (https://github.com/GREsau/schemars/issues/357)
- Support `with`/`into`/`from`/`try_from` container attributes (https://github.com/GREsau/schemars/issues/210 / https://github.com/GREsau/schemars/issues/267)

### Changed

- Use `oneOf` when generating schema for serialized mixed-type sequences (https://github.com/GREsau/schemars/issues/348) - the previous behaviour was to always use `true` schema (i.e. any value) for mixed-type sequences

## [1.0.0-alpha.22] - 2025-06-12

### Added

- Type and const generic params can now be used in `schema_with` attributes, e.g. `#[schemars(schema_with = "func::<T>")]` (https://github.com/GREsau/schemars/pull/426 / https://github.com/GREsau/schemars/issues/375)

## [1.0.0-alpha.21] - 2025-06-09

### Changed

- Improve automatic trait bounds (https://github.com/GREsau/schemars/pull/422 / https://github.com/GREsau/schemars/issues/373)
  - Type params that are only used in skipped fields or `PhantomData` no longer have an unnecessary `JsonSchema` bound automatically added
  - Remove type params from default `schema_name()` whether or not they impl `JsonSchema`. Type params can still be included in the name by specifying them in a rename attribute.
- Recursive references to the root schema type now use `"$ref": "#"` instead of duplicating the entire schema within `$defs`/`definitions` (https://github.com/GREsau/schemars/pull/418 / https://github.com/GREsau/schemars/issues/175)
- Schemas for untagged enum variants no longer have the `"title"` set to the variant name (added in [alpha.19](https://github.com/GREsau/schemars/releases/tag/v1.0.0-alpha.19)) by default, but this behaviour is still available by setting the `untagged_enum_variant_titles` flag on `SchemaSettings`. (https://github.com/GREsau/schemars/pull/421 / https://github.com/GREsau/schemars/issues/420)

## [1.0.0-alpha.20] - 2025-06-01

### Added

- Add `get_mut`, `pointer`, `pointer_mut` methods for easier manipulation of `Schema`s ([#416](https://github.com/GREsau/schemars/issues/416))
- Use `patternProperties` on map schemas where appropriate ([#417](https://github.com/GREsau/schemars/issues/417))
  - ⚠️ `BTreeMap<K,V>`/`HashMap<K,V>` now only implement `JsonSchema` when both `K` and `V` implement `JsonSchema`

### Fixed

- When a struct is `transparent`, don't ignore other attributes ([#415](https://github.com/GREsau/schemars/issues/415))

## [1.0.0-alpha.19] - 2025-05-31

### Added

- Support `#[serde(untagged)]` on individual variants of enums (https://github.com/GREsau/schemars/issues/388 / https://github.com/GREsau/schemars/pull/412)
- Set `"title"` to variant name in schemas for untagged enums/variants (https://github.com/GREsau/schemars/issues/102 / https://github.com/GREsau/schemars/pull/413)

### Changed

- When populating a schema's `"description"` from rust doc comments, trim a single leading space from the comment (https://github.com/GREsau/schemars/pull/407)
- Invalid `with`/`serialize_with` attributes will now fail compilation rather than being silently ignored (https://github.com/GREsau/schemars/pull/410)

### Removed

- Remove the `include_type_name` setting for including `"x-rust-type"` property on generated schemas, since it didn't solve the original feature request. If you have a use-case for that behaviour, please raise an issue in GitHub.

## [0.9.0] - 2025-05-26

This version is identical to `1.0.0-alpha.18`, but is available for those who are unable to unwilling to use a pre-release version.

Those upgrading from Schemars 0.8 may want to consult [the migration guide](https://graham.cool/schemars/migrating/), which also applies when migrating from 0.8 to 0.9.

## [1.0.0-alpha.18] - 2025-05-26

### Added

- `#[schemars(inline)]` attribute for inling schemas when deriving `JsonSchema` (https://github.com/GREsau/schemars/pull/380)
- Implement `JsonSchema` for [jiff](https://crates.io/crates/jiff) 0.2 types, under the optional `jiff02` feature flag (https://github.com/GREsau/schemars/pull/364)
- Add methods to `dyn GenTransform`, allowing to to be used similarly to a `dyn Any`:
  - `fn is<T>(&self) -> bool`
  - `fn downcast_ref<T>(&self) -> Option<&T>`
  - `fn downcast_mut<T>(&mut self) -> Option<&mut T>`
  - `fn downcast<T>(self: Box<Self>) -> Result<Box<T>, Box<Self>>`
- Schemas for `i8`/`i16`/`u8`/`u16` now include `minimum` and `maximum` properties (https://github.com/GREsau/schemars/issues/298)
- `schemars::transform::RestrictFormats` - a `Transform` that removes any `format` values that are not defined by the JSON Schema standard (or explicitly allowed by a custom list). This can be used to remove non-standard `format`s from schemas.
- `SchemaSettings` now has an `include_type_name` flag. When enabled, this includes an `"x-rust-type"` property on generated schemas, set to the name of the schema's associated rust type.

### Changed

- Rename `JsonSchema::always_inline_schema()` to `inline_schema()`, because future attributes may allow particular fields to be uninlined
- MSRV is now 1.74
- `GenTransform::as_any` and `GenTransform::as_any` are deprecated and will be removed before schemars 1.0 becomes stable.
- The generation of nullable schemas (i.e. schemas for `Option<T>`) has been reworked, making them more accurate. The `SchemaSettings::option_nullable` and `SchemaSettings::option_add_null_type` fields are no longer used - instead, generated schemas always include the `"null"` type, but this can be changed to `nullable` by using the new `AddNullable` transform.
- Update OpenAPI 3.0 meta-schema to an active URL (https://github.com/GREsau/schemars/issues/394)
- Change `SchemaSettings::meta_schema` and `SchemaSettings::definitions_path` from `String` to `Cow<'static, str>`, making it easier to construct a `SchemaSettings` in a `const` context.
- The `SchemaGenerator::take_definitions` method now takes an `apply_transforms` flag, which when enabled, will apply the generator's current transforms to each of the schema values in the returned map.

## [0.8.22] - 2025-02-25

### Fixed:

- Fix compatibility with rust 2024 edition (https://github.com/GREsau/schemars/pull/378)

## [1.0.0-alpha.17] - 2024-12-02

### Changed

- For newtype variants of internally-tagged enums, prefer referencing the inner type's schema via `$ref` instead of always inlining the schema (https://github.com/GREsau/schemars/pull/355) _(this change was included in the release notes for 1.0.0-alpha.16, but was accidentally excluded from the published crate)_

## [1.0.0-alpha.16] - 2024-11-25

### Removed (_⚠️ breaking changes ⚠️_)

- the `enumset1`/`enumset` optional dependency has been removed, as its `JsonSchema` impl did not actually match the default serialization format of `EnumSet` (https://github.com/GREsau/schemars/pull/339)

### Changed (_⚠️ breaking changes ⚠️_)

- MSRV is now 1.70
- [The `example` attribute](https://graham.cool/schemars/deriving/attributes/#example) value is now an arbitrary expression, rather than a string literal identifying a function to call. To avoid silent behaviour changes, the expression must not be a string literal where the value can be parsed as a function path - e.g. `#[schemars(example = "foo")]` is now a compile error, but `#[schemars(example = foo())]` is allowed (as is `#[schemars(example = &"foo")]` if you want the the literal string value `"foo"` to be the example).

### Fixed

- The "deserialize" schema for `bytes::Bytes`/`BytesMut` now allows strings, matching the actual deserialize behaviour of the types.
- The schema for `either::Either` now matches the actual serialize/deserialize behaviour of that type.

## [1.0.0-alpha.15] - 2024-09-05

### Added

- `SchemaSettings` now has a `contract` field which determines whether the generated schemas describe how types are serialized or *de*serialized. By default, this is set to `Deserialize`, as this more closely matches the behaviour of previous versions - you can change this to `Serialize` to instead generate schemas describing the type's serialization behaviour (https://github.com/GREsau/schemars/issues/48 / https://github.com/GREsau/schemars/pull/335)

### Changed

- Schemas generated for enums with no variants will now generate `false` (or equivalently `{"not":{}}`), instead of `{"enum":[]}`. This is so generated schemas no longer violate the JSON Schema spec's recommendation that a schema's `enum` array "SHOULD have at least one element".

## [1.0.0-alpha.14] - 2024-08-29

### Added

- Read `#[garde(...)]` attributes as an alternative to `#[validate(...)]` (https://github.com/GREsau/schemars/issues/233 / https://github.com/GREsau/schemars/pull/331). See [the documentation](https://graham.cool/schemars/deriving/attributes/#supported-validatorgarde-attributes) for a full list of supported attributes.

## [1.0.0-alpha.13] - 2024-08-27

### Fixed

- Fix compile errors when using `#[validate(regex(path = *expr))]` attribute

## [1.0.0-alpha.12] - 2024-08-27

### Fixed

- Allow `regex(path = ...)` value to be a non-string expression (https://github.com/GREsau/schemars/issues/302 / https://github.com/GREsau/schemars/pull/328)
- Respect `#[serde(rename_all_fields = ...)]` attribute (https://github.com/GREsau/schemars/issues/273 / https://github.com/GREsau/schemars/pull/304)

### Changed (_⚠️ possibly-breaking changes ⚠️_)

- Invalid attributes that were previously silently ignored (e.g. setting `schema_with` on structs) will now cause compile errors
- Validation attribute parsing has been altered to match the latest version of the validator crate:
  - Remove the `phone` attribute
  - Remove the `required_nested` attribute
  - `regex` and `contains` attributes must now be specified in list form `#[validate(regex(path = ...))]` rather than name/value form `#[validate(regex = ...)]`

## [1.0.0-alpha.11] - 2024-08-24

### Changed

- Values in `#[doc = ...]` and `#[schemars(description = ..., title = ...)]` attributes may now be any arbitrary expression rather than just string literals. (https://github.com/GREsau/schemars/issues/204 / https://github.com/GREsau/schemars/pull/327)
- ⚠️ MSRV is now 1.65 ⚠️

## [1.0.0-alpha.10] - 2024-08-22

### Fixed

- Fix some cases of unsatisfiable schemas generated when flattening enums (https://github.com/GREsau/schemars/pull/325 / https://github.com/GREsau/schemars/issues/164 / https://github.com/GREsau/schemars/issues/165)

## [1.0.0-alpha.9] - 2024-08-21

### Added

- Add rustdoc for `derive(JsonSchema)` macro (https://github.com/GREsau/schemars/issues/322 / https://github.com/GREsau/schemars/issues/322)

## [1.0.0-alpha.8] - 2024-08-21

### Changed

- Replace `schemars::gen` module with `schemars::generate`. This is because `gen` is a reserved keyword in rust 2024, so can only be used as `r#gen`. The `schemars::gen` module is still available for ease of upgrading, but is marked as deprecated and _may_ be removed in the future 1.0.0 release. (https://github.com/GREsau/schemars/issues/306 / https://github.com/GREsau/schemars/pull/323)

## [1.0.0-alpha.7] - 2024-08-19

### Fixed

- Fix behaviour of `flatten` for schemas with `additionalProperties`
- Fix behaviour of `flatten` of multiple enums (https://github.com/GREsau/schemars/issues/165 / https://github.com/GREsau/schemars/pull/320)

## [1.0.0-alpha.6] - 2024-08-17

### Fixed

- Fixed a configuration error that caused rustdoc generation to fail on docs.rs

## [1.0.0-alpha.5] - 2024-08-17

### Added

- Schemars can now be used in `no_std` environments by disabling the new `std` feature flag (which is enabled by default). Schemars still requires an allocator to be available.

## [1.0.0-alpha.4] - 2024-08-17

### Fixed

- Reduce size of MIR output (and improve release-mode compile time) when deriving `JsonSchema` involving applying schema metadata
- Fix `flatten`ing of `serde_json::Value`
- Use absolute import for `Result` in derive output, ignoring any locally imported types called `Result` (https://github.com/GREsau/schemars/pull/307)

## [1.0.0-alpha.3] - 2024-08-10

### Added

- `#[schemars(transform = some::transform)]` for applying arbitrary modifications to generated schemas. `some::transform` must be an expression of type `schemars::transform::Transform` - note that this can be a function with the signature `fn(&mut Schema) -> ()`.
- `SchemaSettings` and `SchemaGenerator` are both now `Send`

### Changed (_⚠️ breaking changes ⚠️_)

- `visit` module and `Visitor` trait have been replace with `transform` and `Transform` respectively. Accordingly, these items have been renamed:
  - `SchemaSettings::visitors` -> `SchemaSettings::transforms`
  - `SchemaSettings::with_visitor` -> `SchemaSettings::with_transform`
  - `SchemaGenerator::visitors_mut` -> `SchemaGenerator::transforms_mut`
  - `GenVisitor` -> `GenTransform`
  - `Visitor::visit_schema` -> `Transform::transform`
  - `visit::visit_schema` -> `transform::transform_subschemas`
- `GenTransform` must also impl `Send`, but no longer needs to impl `Debug`
- Doc comments no longer have newlines collapsed when generating the `description` property (https://github.com/GREsau/schemars/pull/310)

## [1.0.0-alpha.2] - 2024-06-05

### Added

- `#[schemars(extend("key" = value))]` attribute which can be used to add properties (or replace existing properties) in a generated schema (https://github.com/GREsau/schemars/issues/50 / https://github.com/GREsau/schemars/pull/297)
  - Can be set on a struct, enum, or enum variant
  - Value can be any expression that results in a value implementing `Serialize`
  - Value can also be a JSON literal following the rules of `serde_json::json!(value)` macro, i.e. it can interpolate other values that implement `Serialize`

## [1.0.0-alpha.1] - 2024-05-27

### Added

- `json_schema!` macro for creating a custom `Schema`
- Implement `JsonSchema` for [uuid](https://crates.io/crates/uuid) 1.x types, under the optional `uuid1` feature flag
- `SchemaSettings::draft2020_12()` to construct settings conforming to [JSON Schema draft 2020-12](https://json-schema.org/draft/2020-12/release-notes)

### Changed (_⚠️ breaking changes ⚠️_)

- The `Schema` type is now defined as a thin wrapper around a `serde_json::Value`
- The default `SchemaSettings` (used by the `schema_for!()`/`schema_for_value!()` macros and `SchemaGenerator::default()`) now conform to JSON Schema draft 2020-12 instead of draft 7.
- Schemas generated using `SchemaSettings::draft2019_09()` (and `draft2020_12()` and `default()`) now use `$defs` instead of `definitions`. While using `definitions` is allowed by the spec, `$defs` is the preferred property for storing reusable schemas.
- `JsonSchema::schema_name()` now returns `Cow<'static, str>` instead of `String`
- `JsonSchema::is_referenceable()` has been removed, and replaced with the more clearly-named `JsonSchema::always_inline_schema()` (which should returns the **opposite** value to what `is_referenceable` returned!)
- The `SchemaGenerator.definitions` field is now a `serde_json::Map<String, Value>`
- Macros/functions that previously returned a `RootSchema` now return a `Schema` instead
- All optional dependencies are now suffixed by their version:
  - `chrono` is now `chrono04`
  - `either` is now `either1`
  - `smallvec` is now `smallvec1`
  - `url` is now `url2`
  - `bytes` is now `bytes1`
  - `rust_decimal` is now `rust_decimal1`
  - `enumset` is now `enumset1`
  - `smol_str` is now `smol_str02`
  - `semver` is now `semver1`
  - `indexmap2`, `arrayvec07` and `bigdecimal04` are unchanged

### Removed (_⚠️ breaking changes ⚠️_)

- Removed deprecated `SchemaGenerator` methods `make_extensible`, `schema_for_any` and `schema_for_none`
- Removed the `schema` module
  - The `Schema` type is now accessible from the crate root (i.e. `schemars::Schema` instead of `schemars::schema::Schema`)
  - All other types that were in the module have been removed:
    - `RootSchema`
    - `SchemaObject`
    - `Metadata`
    - `SubschemaValidation`
    - `NumberValidation`
    - `StringValidation`
    - `ArrayValidation`
    - `ObjectValidation`
    - `InstanceType`
    - `SingleOrVec`
- Removed `schemars::Set` and `schemars::Map` type aliases
- Removed the `impl_json_schema` feature flag - `JsonSchema` is now always implemented on `Schema`
- Remove methods `visit_schema_object` and `visit_root_schema` from the `Visitor` trait (`visit_schema` is unchanged)
  - Visitors that previously defined `visit_schema_object` should instead define `visit_schema` and use an `if let Some(obj) = schema.as_object_mut()` or similar construct
- Old versions of optional dependencies have been removed - all of these have newer versions (shown in brackets) which are supported by schemars
  - `indexmap` (consider using `indexmap2`)
  - `uuid08` (consider using `uuid1`)
  - `arrayvec05` (consider using `arrayvec07`)
  - `bigdecimal03` (consider using `bigdecimal04`)
- Remove the retain_examples field from SetSingleExample, which is now a unit struct

## [0.8.21] - 2024-05-23

### Fixed:

- Fix `null` default not being set on generated schemas (https://github.com/GREsau/schemars/issues/295 / https://github.com/GREsau/schemars/pull/296)

## [0.8.20] - 2024-05-18

### Fixed:

- Revert unintentional change in behaviour when combining `default` and `required` attributes (https://github.com/GREsau/schemars/issues/292)

## [0.8.19] - 2024-05-06

### Fixed:

- Regression that caused a compile error when deriving `JsonSchema` on an enum with no variants (https://github.com/GREsau/schemars/issues/287)

## [0.8.18] - 2024-05-06

### Fixed:

- Reduce size of MIR output (and improve release-mode compile time) when deriving `JsonSchema` on enums (https://github.com/GREsau/schemars/pull/266 / https://github.com/GREsau/schemars/pull/286)

## [0.8.17] - 2024-04-28

### Changed:

- Update to syn 2.0, which should improve compile times in many cases (https://github.com/GREsau/schemars/pull/281)

## [0.8.16] - 2023-11-11

### Fixed:

- Reduce size of MIR output (and improve release-mode compile time) when deriving `JsonSchema`

## [0.8.15] - 2023-09-17

### Added:

- Implement `JsonSchema` for `BigDecimal` from `bigdecimal` 0.4 (https://github.com/GREsau/schemars/pull/237)

## [0.8.14] - 2023-09-17

### Added:

- Add `#[schemars(inner(...)]` attribute to specify schema for array items (https://github.com/GREsau/schemars/pull/234)

### Changed:

- New optional associated function on `JsonSchema` trait: `schema_id()`, which is similar to `schema_name()`, but does not have to be human-readable, and defaults to the type name including module path. This allows schemars to differentiate between types with the same name in different modules/crates (https://github.com/GREsau/schemars/issues/62 / https://github.com/GREsau/schemars/pull/247)

### Fixed:

- Schemas for `rust_decimal::Decimal` and `bigdecimal::BigDecimal` now match how those types are serialized by default, i.e. as numeric strings (https://github.com/GREsau/schemars/pull/248)

## [0.8.13] - 2023-08-28

### Added:

- Implement `JsonSchema` for `semver::Version` (https://github.com/GREsau/schemars/pull/195 / https://github.com/GREsau/schemars/pull/238)
- Include const generics in generated schema names (https://github.com/GREsau/schemars/pull/179 / https://github.com/GREsau/schemars/pull/239)
- Implement `JsonSchema` for types from indexmap v2 (https://github.com/GREsau/schemars/pull/226 / https://github.com/GREsau/schemars/pull/240)
- Implement `JsonSchema` for `serde_json::value::RawValue` (https://github.com/GREsau/schemars/pull/183)

### Changed:

- Minimum supported rust version is now 1.60.0

## [0.8.12] - 2023-02-26

### Added:

- Implement `JsonSchema` for `smol_str::SmolStr` (https://github.com/GREsau/schemars/pull/72)

### Changed:

- Change `serde_json` dependency min version to 1.0.25 (was 1.0.0) (https://github.com/GREsau/schemars/pull/192)

## [0.8.11] - 2022-10-02

### Added:

- Replace auto-inferred trait bounds with bounds specified in `#[schemars(bound = "...")]` attribute

### Changed:

- Derived `JsonSchema` now respects attributes on unit enum variants (https://github.com/GREsau/schemars/pull/152)
- Minimum supported rust version is now 1.45.0

## [0.8.10] - 2022-05-17

- Undo "Support generic default values in default attributes (https://github.com/GREsau/schemars/pull/83)" as it inadvertently introduced a breaking change (https://github.com/GREsau/schemars/issues/144)

## [0.8.9] - 2022-05-16

### Added:

- ~~Support generic default values in `default` attributes (https://github.com/GREsau/schemars/pull/83)~~
  - **This inadvertently introduced a breaking change and was removed in 0.8.10**
- Add missing MIT licence text for usage of code from regex_syntax crate (https://github.com/GREsau/schemars/pull/132)
- Support uuid v1 and arrayvec 0.7 via feature flags `uuid1` and `arrayvec07` (https://github.com/GREsau/schemars/pull/142)
  - This also adds `uuid08` and `arrayvec05` feature flags for the previously supported versions of these crates. The existing `uuid` and `arrayvec` flags are still supported for backward-compatibility, but they are **deprecated**.
  - Similarly, `indexmap1` feature flag is added, and `indexmap` flag is **deprecated**.

## [0.8.8] - 2021-11-25

### Added:

- Implement `JsonSchema` for types from `rust_decimal` and `bigdecimal` crates (https://github.com/GREsau/schemars/pull/101)

### Fixed:

- Fixes for internally tagged enums and flattening additional_properties (https://github.com/GREsau/schemars/pull/113)

## [0.8.7] - 2021-11-14

### Added:

- Implement `JsonSchema` for `EnumSet` (https://github.com/GREsau/schemars/pull/92)

### Fixed:

- Do not cause compile error when using a default value that doesn't implement `Serialize` (https://github.com/GREsau/schemars/issues/115)

## [0.8.6] - 2021-09-26

### Changed:

- Use `oneOf` instead of `anyOf` for enums when possible (https://github.com/GREsau/schemars/issues/108)

## [0.8.5] - 2021-09-20

### Fixed:

- Allow fields with plain `#[validate]` attributes (https://github.com/GREsau/schemars/issues/109)

## [0.8.4] - 2021-09-19

### Added:

- `#[schemars(schema_with = "...")]` attribute can now be set on enum variants.
- Deriving JsonSchema will now take into account `#[validate(...)]` attributes, compatible with the [validator](https://github.com/Keats/validator) crate (https://github.com/GREsau/schemars/pull/78)

## [0.8.3] - 2021-04-05

### Added:

- Support for `#[schemars(crate = "...")]` attribute to allow deriving JsonSchema when the schemars crate is aliased to a different name (https://github.com/GREsau/schemars/pull/55 / https://github.com/GREsau/schemars/pull/80)
- Implement `JsonSchema` for `bytes::Bytes` and `bytes::BytesMut` (https://github.com/GREsau/schemars/pull/68)

### Fixed:

- Fix deriving JsonSchema on types defined inside macros (https://github.com/GREsau/schemars/issues/59 / https://github.com/GREsau/schemars/issues/66 / https://github.com/GREsau/schemars/pull/79)

## [0.8.2] - 2021-03-27

### Added:

- Enable generating a schema from any serializable value using `schema_for_value!(...)` macro or `SchemaGenerator::root_schema_for_value()`/`SchemaGenerator::into_root_schema_for_value()` methods (https://github.com/GREsau/schemars/pull/75)
- `#[derive(JsonSchema_repr)]` can be used on C-like enums for generating a serde_repr-compatible schema (https://github.com/GREsau/schemars/pull/76)
- Implement `JsonSchema` for `url::Url` (https://github.com/GREsau/schemars/pull/63)

## [0.8.1] - 2021-03-23

### Added:

- `SchemaGenerator::definitions_mut()` which returns a mutable reference to the generator's schema definitions
- Implement `JsonSchema` for slices

### Changed:

- Minimum supported rust version is now 1.37.0
- Deriving JsonSchema on enums now sets `additionalProperties` to false on generated schemas wherever serde doesn't accept unknown properties. This includes non-unit variants of externally tagged enums, and struct-style variants of all enums that have the `deny_unknown_fields` attribute.
- Schemas for HashSet and BTreeSet now have `uniqueItems` set to true (https://github.com/GREsau/schemars/pull/64)

### Fixed

- Fix use of `#[serde(transparent)]` in combination with `#[schemars(with = ...)]` (https://github.com/GREsau/schemars/pull/67)
- Fix clippy `field_reassign_with_default` warning in schemars_derive generated code in rust <1.51 (https://github.com/GREsau/schemars/pull/65)
- Prevent stack overflow when using `inline_subschemas` with recursive types

## [0.8.0] - 2020-09-27

### Added:

- `visit::Visitor`, a trait for updating a schema and all schemas it contains recursively. A `SchemaSettings` can now contain a list of visitors.
- `into_object()` method added to `Schema` as a shortcut for `into::<SchemaObject>()`
- Preserve order of schema properties under `preserve_order` feature flag (https://github.com/GREsau/schemars/issues/32)
- `SchemaGenerator::take_definitions()` which behaves similarly to the now-removed `into_definitions()` method but without consuming the generator
- `SchemaGenerator::visitors_mut()` which returns an iterator over a generator's settings's visitors
- `SchemaSettings::inline_subschemas` - enforces inlining of all subschemas instead of using references (https://github.com/GREsau/schemars/issues/44)

### Removed (**BREAKING CHANGES**):

- `SchemaSettings::bool_schemas` - this has been superseded by the `ReplaceBoolSchemas` visitor
- `SchemaSettings::allow_ref_siblings` - this has been superseded by the `RemoveRefSiblings` visitor
- `SchemaSettings` no longer implements `PartialEq`
- `SchemaGenerator::into_definitions()` - this has been superseded by `SchemaGenerator::take_definitions()`

### Changed:

- **BREAKING CHANGE** Minimum supported rust version is now 1.36.0

### Fixed:

- **BREAKING CHANGE** unknown items in `#[schemars(...)]` attributes now cause a compilation error (https://github.com/GREsau/schemars/issues/18)

### Deprecated:

- `make_extensible`, `schema_for_any`, and `schema_for_none` methods on `SchemaGenerator`

## [0.7.6] - 2020-05-17

### Added:

- `#[schemars(example = "...")]` attribute for setting examples on generated schemas (https://github.com/GREsau/schemars/issues/23)

## [0.7.5] - 2020-05-17

### Added:

- Setting `#[deprecated]` attribute will now cause generated schemas to have the `deprecated` property set to `true`
- Respect `#[serde(transparent)]` attribute (https://github.com/GREsau/schemars/issues/17)
- `#[schemars(title = "...", description = "...")]` can now be used to set schema title/description. If present, these values will be used instead of doc comments (https://github.com/GREsau/schemars/issues/13)

### Changed:

- schemars_derive is now an optional dependency, but included by default

## [0.7.4] - 2020-05-16

### Added:

- If a struct is annotated with `#[serde(deny_unknown_fields)]`, generated schema will have `additionalProperties` set to `false` (https://github.com/GREsau/schemars/pull/30)
- Set `type` property to `string` on simple enums (https://github.com/GREsau/schemars/issues/28)

## [0.7.3] - 2020-05-15

### Added:

- `#[schemars(schema_with = "...")]` attribute can be set on variants and fields. This allows you to specify another function which returns the schema you want, which is particularly useful on fields of types that don't implement the JsonSchema trait (https://github.com/GREsau/schemars/issues/15)

### Fixed

- `#[serde(with = "...")]`/`#[schemars(with = "...")]` attributes on enum variants are now respected
- Some compiler errors generated by schemars_derive should now have more accurate spans

## [0.7.2] - 2020-04-30

### Added:

- Enable deriving JsonSchema on adjacent tagged enums (https://github.com/GREsau/schemars/issues/4)

## [0.7.1] - 2020-04-11

### Added:

- Added `examples` (https://tools.ietf.org/html/draft-handrews-json-schema-validation-02#section-9.5) to `Metadata`

### Fixed

- Fixed a bug in schemars_derive causing a compile error when the `default`, `skip_serializing_if`, and `serialize_with`/`with` attributes are used together (https://github.com/GREsau/schemars/issues/26)

## [0.7.0] - 2020-03-24

### Changed:

- **BREAKING CHANGE** - `SchemaSettings` can no longer be created using struct initialization syntax. Instead, if you need to use custom schema settings, you can use a constructor function and either:
  - assign it to a `mut` variable and modify its public fields
  - call the `with(|s| ...)` method on the settings and modify the settings inside the closure/function (as in the custom_settings.rs example)

### Fixed:

- When deriving `JsonSchema` on structs, `Option<T>` struct fields are no longer included in the list of required properties in the schema (https://github.com/GREsau/schemars/issues/11)
- Fix deriving `JsonSchema` when a non-std `String` type is in scope (https://github.com/GREsau/schemars/pull/19)
- This will now compile: `#[schemars(with="()")]`

### Added:

- Added `allow_ref_siblings` setting to `SchemaSettings`. When enabled, schemas with a `$ref` property may have other properties set.
- Can create JSON Schema 2019-09 schemas using `SchemaSettings::draft2019_09()` (which enables `allow_ref_siblings`)

## [0.6.5] - 2019-12-29

### Added:

- Implemented `JsonSchema` on types from `smallvec` and `arrayvec` (as optional dependencies)

## [0.6.4] - 2019-12-27

### Added:

- Implemented `JsonSchema` on types from `indexmap`, `either` and `uuid` (as optional dependencies)

### Changed

- Remove trait bounds from Map/Set JsonSchema impls. They are unnecessary as we never create/use any instances of these types.

## [0.6.3] - 2019-12-27

- No actual code changes - this version was just published to fix broken README on crates.io

## [0.6.2] - 2019-12-27

### Added:

- Documentation website available at https://graham.cool/schemars/!

### Changed:

- Rename `derive_json_schema` to `impl_json_schema`. `derive_json_schema` is still available for backward-compatibility, but will be removed in a future version.
- Improve schema naming for deriving on remote types. A `#[serde(remote = "Duration")]` attribute is now treated similarly to `#[serde(rename = "Duration")]`.
- Ensure root schemas do not have a `$ref` property. If necessary, wrap the `$ref` in an `allOf`.

## [0.6.1] - 2019-12-09

### Fixed:

- Fix a compile error that can occur when deriving `JsonSchema` from a project that doesn't reference serde_json

## [0.6.0] - 2019-12-09

### Added:

- When deriving `JsonSchema`, the schema's `title` and `description` are now set from `#[doc]` comments (https://github.com/GREsau/schemars/issues/7)
- When deriving `JsonSchema` on structs using a `#[serde(default)]` attribute, the schema's properties will now include `default`, unless the default value is skipped by the field's `skip_serializing_if` function (https://github.com/GREsau/schemars/issues/6)

### Changed:

- When the `option_nullable` setting is enabled (e.g. for openapi 3), schemas for `Option<T>` will no longer inline `T`'s schema when it should be referenceable.

## [0.5.1] - 2019-10-30

### Fixed:

- Added missing doc comment for `title` schema property

## [0.5.0] - 2019-10-30

### Added:

- Implemented `JsonSchema` for more standard library types (https://github.com/GREsau/schemars/issues/3)

### Changed:

- Unsigned integer types (usize, u8 etc.) now have their [`minimum`](https://json-schema.org/draft/2019-09/json-schema-validation.html#rfc.section.6.2.4) explicitly set to zero
- Made prepositions/conjunctions in generated schema names lowercase
  - e.g. schema name for `Result<MyStruct, Vec<String>>` has changed from "Result_Of_MyStruct_Or_Array_Of_String" to "Result_of_MyStruct_or_Array_of_String"
- Some provided `JsonSchema` implementations with the same `type` but different `format`s (e.g. `i8` and `usize`) used the `type` as their name. They have now been updated to use `format` as their name.
  - Previously, schema generation would incorrectly assume types such as `MyStruct<i8>` and `MyStruct<usize>` were identical, and give them a single schema definition called `MyStruct_for_Integer` despite the fact they should have different schemas. Now they will each have their own schema (`MyStruct_for_i8` and `MyStruct_for_usize` respectively).
