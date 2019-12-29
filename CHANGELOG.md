# Changelog

## [0.7.0-alpha-1] - 2019-12-29
### Changed:
- **BREAKING CHANGE** - `SchemaSettings` can no longer be created using struct initialization syntax. Instead, if you need to use custom schema settings, you can use a constructor function and either:
    - assign it to a `mut` variable and modify its public fields
    - call the `with(|s| ...)` method on the settings and modify the settings inside the closure/function (as in the custom_settings.rs example)
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