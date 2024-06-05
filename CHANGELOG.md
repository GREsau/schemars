# Changelog

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
- `JsonSchema::is_referenceable()` has been removed, and replaced with the more clearly-named `JsonSchema::always_inline()` (which should returns the **opposite** value to what `is_referenceable` returned!)
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
