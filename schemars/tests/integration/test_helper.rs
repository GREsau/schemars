use jsonschema::JSONSchema as CompiledSchema;
use schemars::{
    generate::{Contract, SchemaSettings},
    JsonSchema, Schema,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use snapbox::IntoJson;
use std::{
    any::type_name, borrow::Borrow, cell::OnceCell, f64, marker::PhantomData, path::Path,
    sync::OnceLock,
};

pub struct TestHelper<T: JsonSchema> {
    settings: SchemaSettings,
    name: String,
    phantom: PhantomData<T>,
    // TODO once MSRV has reached 1.80, replace these with LazyLock
    de_schema: OnceCell<Schema>,
    ser_schema: OnceCell<Schema>,
    de_schema_compiled: OnceCell<CompiledSchema>,
    ser_schema_compiled: OnceCell<CompiledSchema>,
}

impl<T: JsonSchema> TestHelper<T> {
    /// Should be used via the `test!(SomeType)` macro
    pub fn new(name: String, settings: SchemaSettings) -> Self {
        Self {
            settings,
            name,
            phantom: PhantomData,
            de_schema: OnceCell::new(),
            ser_schema: OnceCell::new(),
            de_schema_compiled: OnceCell::new(),
            ser_schema_compiled: OnceCell::new(),
        }
    }

    /// Checks the generated schema against the saved schema in the snapshots directory, for manual verification of changes.
    ///
    /// Run tests with the SNAPSHOTS env var set to "overwrite" to overwrite any changed snapshots.
    pub fn assert_snapshot(&self) -> &Self {
        let de_path = format!("tests/integration/snapshots/{}.de.json", self.name);
        snapbox::assert_data_eq!(
            self.de_schema().into_json(),
            snapbox::Data::read_from(Path::new(&de_path), None).raw()
        );

        let ser_path = format!("tests/integration/snapshots/{}.ser.json", self.name);
        snapbox::assert_data_eq!(
            self.ser_schema().into_json(),
            snapbox::Data::read_from(Path::new(&ser_path), None).raw()
        );

        self
    }

    /// Checks that the schema generated for this type is identical to that of another type.
    pub fn assert_identical<T2: JsonSchema>(&self) -> &Self {
        snapbox::assert_data_eq!(
            self.de_schema().into_json(),
            self.schema_for::<T2>(Contract::Deserialize)
                .into_json()
                .raw()
        );
        snapbox::assert_data_eq!(
            self.ser_schema().into_json(),
            self.schema_for::<T2>(Contract::Serialize).into_json().raw()
        );

        let t = type_name::<T>();
        let t2 = type_name::<T2>();
        assert_eq!(
            T::schema_name(),
            T2::schema_name(),
            "`{t}` and `{t2}` have identical schemas, so should have the same schema_name"
        );

        assert_eq!(
            T::schema_id(),
            T2::schema_id(),
            "`{t}` and `{t2}` have identical schemas, so should have the same schema_id"
        );

        assert_eq!(
            T::always_inline_schema(),
            T2::always_inline_schema(),
            "`{t}` and `{t2}` have identical schemas, so should have the same always_inline_schema"
        );

        self
    }

    pub fn custom(&self, assertion: impl Fn(&Schema, Contract)) {
        assertion(self.de_schema(), Contract::Deserialize);
        assertion(self.ser_schema(), Contract::Serialize);
    }

    fn schema_for<T2: JsonSchema>(&self, contract: Contract) -> Schema {
        self.settings
            .clone()
            .with(|s| s.contract = contract)
            .into_generator()
            .into_root_schema_for::<T2>()
    }

    fn de_schema(&self) -> &Schema {
        self.de_schema
            .get_or_init(|| self.schema_for::<T>(Contract::Deserialize))
    }

    fn ser_schema(&self) -> &Schema {
        self.ser_schema
            .get_or_init(|| self.schema_for::<T>(Contract::Serialize))
    }

    fn de_schema_validate(&self, instance: &Value) -> bool {
        self.de_schema_compiled
            .get_or_init(|| compile_schema(self.de_schema()))
            // Can't use `.validate(instance)` due to https://github.com/Stranger6667/jsonschema-rs/issues/496
            .validate(instance)
            .is_ok()
    }

    fn ser_schema_validate(&self, instance: &Value) -> bool {
        self.ser_schema_compiled
            .get_or_init(|| compile_schema(self.ser_schema()))
            // Can't use `.validate(instance)` due to https://github.com/Stranger6667/jsonschema-rs/issues/496
            .validate(instance)
            .is_ok()
    }
}

fn compile_schema(schema: &Schema) -> CompiledSchema {
    use jsonschema::Draft;

    let meta_schema = schema.get("$schema").and_then(Value::as_str).unwrap_or("");
    let mut options = CompiledSchema::options();
    options.should_validate_formats(true);

    if meta_schema.contains("draft-07") {
        options.with_draft(Draft::Draft7);
    } else if meta_schema.contains("2019-09") {
        options.with_draft(Draft::Draft201909);
    } else if meta_schema.contains("2020-12") {
        options.with_draft(Draft::Draft202012);
    };

    options.compile(schema.as_value()).expect("valid schema")
}

impl<T: JsonSchema + Serialize + for<'de> Deserialize<'de>> TestHelper<T> {
    /// Checks that the "serialize" schema allows the given sample values when serialized to JSON
    /// and, if the value can then be deserialized, that the "deserialize" schema also allows it.
    pub fn assert_allows_ser_roundtrip(&self, samples: impl IntoIterator<Item = T>) -> &Self {
        for sample in samples {
            let json = serde_json::to_value(sample).unwrap();
            assert!(
                self.ser_schema_validate(&json),
                "serialize schema should allow serialized value: {json}"
            );

            if T::deserialize(&json).is_ok() {
                assert!(
                    self.de_schema_validate(&json),
                    "deserialize schema should allow value accepted by deserialization: {json}"
                );
            } else {
                assert!(
                    !self.de_schema_validate(&json),
                    "deserialize schema should reject undeserializable value: {json}"
                );
            }
        }

        self
    }

    /// Checks that the "deserialize" schema allow the given sample values, and the "serialize"
    /// schema allows the value obtained from deserializing then re-serializing the sample values
    /// (only for values that can actually be serialized).
    ///
    /// This is intended for types that have different serialize/deserialize schemas, or when you
    /// want to test specific values that are valid for deserialization but not for serialization.
    pub fn assert_allows_de_roundtrip(
        &self,
        samples: impl IntoIterator<Item = impl Borrow<Value>>,
    ) -> &Self {
        for sample in samples {
            let sample = sample.borrow();
            let Ok(deserialized) = T::deserialize(sample) else {
                panic!(
                    "expected deserialize to succeed for {}: {sample}",
                    type_name::<T>()
                )
            };

            assert!(
                self.de_schema_validate(sample),
                "deserialize schema should allow value accepted by deserialization: {sample}"
            );

            if let Ok(serialized) = serde_json::to_value(&deserialized) {
                assert!(
                    self.ser_schema_validate(&serialized),
                    "serialize schema should allow serialized value: {serialized}"
                );
            }
        }

        self
    }

    /// Checks that the "deserialize" schema allows only the given sample values that successfully
    /// deserialize.
    ///
    /// This is intended to be given a range of values (see `arbitrary_values`), allowing limited
    /// fuzzing.
    pub fn assert_matches_de_roundtrip(
        &self,
        samples: impl IntoIterator<Item = impl Borrow<Value>>,
    ) -> &Self {
        for value in samples {
            let value = value.borrow();

            match T::deserialize(value) {
                Ok(deserialized) => {
                    assert!(
                        self.de_schema_validate(value),
                        "deserialize schema should allow value accepted by deserialization: {value}"
                    );

                    if let Ok(serialized) = serde_json::to_value(&deserialized) {
                        assert!(
                            self.ser_schema_validate(&serialized),
                            "serialize schema should allow serialized value: {serialized}"
                        );
                    }
                }
                Err(_) => {
                    assert!(
                        !self.de_schema_validate(value),
                        "deserialize schema should reject invalid value: {value}"
                    );

                    // This assertion isn't necessarily valid in the general case but it would be
                    // odd (though not necessarily wrong) for it to fail. If this does ever fail
                    // a case that should be legitimate, then this assert can be removed/weakened.
                    assert!(
                        !self.ser_schema_validate(value),
                        "serialize schema should reject invalid value: {value}"
                    );
                }
            }
        }

        self
    }

    /// Checks that the "deserialize" schema does not allow any of the given sample values.
    ///
    /// While `assert_matches_de_roundtrip()` would also work in this case, `assert_rejects_de()`
    /// has the advantage that it also verifies that the test case itself is actually covering the
    /// failure case as intended.
    pub fn assert_rejects_de(&self, values: impl IntoIterator<Item = impl Borrow<Value>>) -> &Self {
        for value in values {
            let value = value.borrow();

            assert!(
                T::deserialize(value).is_err(),
                "invalid test case - expected deserialize to fail for {}: {value}",
                type_name::<T>()
            );

            assert!(
                !self.de_schema_validate(value),
                "deserialize schema should reject invalid value: {value}"
            );
        }

        self
    }

    /// Checks that both the "serialize" and "deserialize" schema allow the type's default value
    /// when serialized to JSON.
    pub fn assert_allows_ser_roundtrip_default(&self) -> &Self
    where
        T: Default,
    {
        self.assert_allows_ser_roundtrip([T::default()])
    }
}

/// Returns an iterator over an selection of arbitrary JSON values.
///
/// This is intended to be used as `test!(...).assert_matches_de_roundtrip(arbitrary_values())`
pub fn arbitrary_values() -> impl Iterator<Item = &'static Value> {
    fn primitives() -> impl Iterator<Item = Value> {
        [
            Value::Null,
            false.into(),
            true.into(),
            0.into(),
            255.into(),
            (-1).into(),
            u64::MAX.into(),
            f64::consts::PI.into(),
            "".into(),
            "0".into(),
            "3E8".into(),
            "\tPâté costs “£1”\0".into(),
            Value::Array(Default::default()),
            Value::Object(Default::default()),
        ]
        .into_iter()
    }

    // TODO once MSRV has reached 1.80, replace this with LazyLock
    static VALUES: OnceLock<Vec<Value>> = OnceLock::new();

    VALUES
        .get_or_init(|| {
            Vec::from_iter(
                primitives()
                    .chain(primitives().map(|p| json!([p])))
                    .chain(primitives().map(|p| json!({"key": p}))),
            )
        })
        .iter()
}

/// Returns an iterator over an selection of arbitrary JSON values, except for value that match
/// the given filter predicate.
///
/// This is to handle known cases of schemas not matching the actual deserialize behaviour.
pub fn arbitrary_values_except(
    filter: impl Fn(&Value) -> bool,
    _reason: &str,
) -> impl Iterator<Item = &'static Value> {
    arbitrary_values().filter(move |v| !filter(v))
}
