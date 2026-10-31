//! The IR files that `qualify:moonbit` sends to the MoonBit CLI must stay equal
//! to the projection fixtures that the crate tests use.
//!
//! `MORPHIR_WRITE_FIXTURES=1 cargo test -p extension-guest-check --test fixtures`
//! rewrites them.

use std::path::PathBuf;

use morphir_projection::testing::classic;
use serde_json::Value;

fn check(name: &str, expected: Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    let text = format!(
        "{}\n",
        serde_json::to_string_pretty(&expected).expect("JSON")
    );
    if std::env::var_os("MORPHIR_WRITE_FIXTURES").is_some() {
        std::fs::write(&path, &text).expect("the fixture is written");
    }
    let actual = std::fs::read_to_string(&path).expect("the fixture exists");
    assert_eq!(
        actual, text,
        "{name} differs; rerun with MORPHIR_WRITE_FIXTURES=1"
    );
}

#[test]
fn the_classic_schema_library_file_is_current() {
    check(
        "classic-schema-library-v3.json",
        classic::classic_schema_library(),
    );
}

/// Byte-equal, not reformatted: the file is an upstream fixture (finos/morphir-moonbit
/// `apps/morphir-workbench/fixtures/typed-pricing.json` at `fd4026e`), kept as written.
#[test]
fn the_typed_pricing_file_is_current() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/typed-pricing-v4.json");
    let expected =
        include_str!("../../morphir-openapi-extension/tests/fixtures/typed-pricing.json");
    if std::env::var_os("MORPHIR_WRITE_FIXTURES").is_some() {
        std::fs::write(&path, expected).expect("the fixture is written");
    }
    let actual = std::fs::read_to_string(&path).expect("the fixture exists");
    assert!(
        actual == expected,
        "typed-pricing-v4.json differs; rerun with MORPHIR_WRITE_FIXTURES=1"
    );
}

#[test]
fn the_canonical_customer_file_is_current() {
    let canonical: Value = serde_json::from_str(include_str!(
        "../../morphir-avro-extension/tests/fixtures/canonical-customer-v4.json"
    ))
    .expect("JSON");
    check("canonical-customer-v4.json", canonical);
}
