# Changelog

## [0.8.3] - **In-dev**
### Added:
- Support for `#[schemars(crate = "...")]` attribute to allow deriving JsonSchema when the schemars crate is aliased to a different name (https://github.com/GREsau/schemars/pull/55 / https://github.com/GREsau/schemars/pull/80)

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
