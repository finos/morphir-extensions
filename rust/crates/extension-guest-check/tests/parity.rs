//! The released module must answer as the native crate does.
//!
//! These tests need the release modules. Build them first, then run the ignored
//! tests (`mise run rust:guest-check` does both):
//!
//! `cargo build --release --workspace --target wasm32-unknown-unknown`
//! `cargo test -p extension-guest-check -- --ignored`
//!
//! `EXTENSION_GUEST_DIR` selects another directory with the modules.

use std::collections::HashMap;
use std::path::PathBuf;

use extension_guest_check::Guest;
use morphir_avro_extension::AvroExtension;
use morphir_extension_sdk::claims::CapabilityClaimSet;
use morphir_extension_sdk::{Backend, GenerateRequest};
use morphir_openapi_extension::OpenApiExtension;
use morphir_projection::testing::{classic, v4};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};

fn module(file: &str) -> PathBuf {
    let dir = std::env::var_os("EXTENSION_GUEST_DIR").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/wasm32-unknown-unknown/release")
        },
        PathBuf::from,
    );
    let path = dir.join(file);
    assert!(
        path.is_file(),
        "{} is missing; build the release modules first",
        path.display()
    );
    path
}

fn canonical_customer() -> Value {
    serde_json::from_str(include_str!(
        "../../morphir-avro-extension/tests/fixtures/canonical-customer-v4.json"
    ))
    .expect("the fixture is JSON")
}

/// Every IR input of the crate tests, and two that the guest must refuse.
fn models() -> Vec<(&'static str, Value)> {
    vec![
        ("v4 customer application", v4::v4_customer_application()),
        ("v4 customer library", v4::v4_customer_library()),
        ("v4 customer specs", v4::v4_customer_specs()),
        ("v4 incomplete library", v4::v4_incomplete_library()),
        ("canonical customer v4", canonical_customer()),
        (
            "classic customer library",
            classic::classic_customer_library(),
        ),
        ("classic schema library", classic::classic_schema_library()),
        (
            "classic colliding names",
            classic::classic_colliding_names_library(),
        ),
        (
            "classic function field",
            classic::classic_function_field_library(),
        ),
        ("missing format version", json!({ "distribution": null })),
        (
            "format version 5",
            json!({ "formatVersion": 5, "distribution": null }),
        ),
    ]
}

fn to_map(options: &Value) -> HashMap<String, Value> {
    options
        .as_object()
        .expect("options are an object")
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Run every model with every option set through the module in one session and
/// through the native backend, and compare the answers.
fn assert_parity<B: Backend>(native: &B, file: &str, targets: &[&str], option_sets: &[Value]) {
    let mut guest = Guest::load(&module(file)).expect("the module loads");
    guest.initialize().expect("the session starts");
    let mut generated = 0;
    for target in targets {
        for options in option_sets {
            for (name, ir) in models() {
                let expected = native
                    .generate(GenerateRequest {
                        ir: ir.clone(),
                        target: (*target).into(),
                        options: to_map(options),
                    })
                    .map(|result| serde_json::to_value(result).expect("the result serializes"));
                let actual = guest.generate(ir, target, options.clone());
                match (expected, actual) {
                    (Ok(expected), Ok(actual)) => {
                        assert_eq!(actual, expected, "{target} {options} {name}");
                        if actual["success"] == json!(true) {
                            generated += 1;
                        }
                    }
                    (Err(_), Err(_)) => {}
                    (expected, actual) => {
                        panic!("{target} {options} {name}: native {expected:?}, module {actual:?}")
                    }
                }
            }
        }
    }
    guest.shutdown().expect("the session ends");
    assert!(generated > 0, "no case generated artifacts");
}

#[test]
#[ignore = "requires the release modules"]
fn avro_module_answers_as_the_native_crate() {
    assert_parity(
        &AvroExtension,
        "morphir_avro_extension.wasm",
        &["avro"],
        &[
            json!({}),
            json!({ "representation": "idl" }),
            json!({ "representation": "json", "projection": "protocol-entry-points" }),
            json!({ "representation": "idl", "projection": "protocol-public" }),
            json!({ "unsupported": "warn-and-skip" }),
            json!({ "dependencies": "self-contained", "representation": "idl" }),
            json!({ "typo": true }),
        ],
    );
}

#[test]
#[ignore = "requires the release modules"]
fn openapi_module_answers_as_the_native_crate() {
    assert_parity(
        &OpenApiExtension,
        "morphir_openapi_extension.wasm",
        &["openapi", "json-schema"],
        &[
            json!({}),
            json!({ "version": "3.0" }),
            json!({ "projection": "operations-entry-points" }),
            json!({ "projection": "operations-public" }),
            json!({ "unsupported": "warn-and-skip" }),
            json!({ "unsupported": "ignore" }),
            json!({ "version": "2.0" }),
        ],
    );
}

/// Read the claims from the module and check them against the committed file.
fn assert_claims(file: &str, expected: &str) {
    let claims = Guest::load(&module(file))
        .expect("the module loads")
        .describe()
        .expect("describe answers");
    serde_json::from_value::<CapabilityClaimSet>(claims.clone()).expect("a valid claim set");
    let expected: Value = serde_json::from_str(expected).expect("the expected claims are JSON");
    assert_eq!(
        claims, expected,
        "the claims of {file} changed; update claims/expected.json of its crate"
    );
}

#[test]
#[ignore = "requires the release modules"]
fn avro_claims_equal_the_committed_claims() {
    assert_claims(
        "morphir_avro_extension.wasm",
        include_str!("../../morphir-avro-extension/claims/expected.json"),
    );
}

#[test]
#[ignore = "requires the release modules"]
fn openapi_claims_equal_the_committed_claims() {
    assert_claims(
        "morphir_openapi_extension.wasm",
        include_str!("../../morphir-openapi-extension/claims/expected.json"),
    );
}

#[test]
#[ignore = "requires the release modules"]
fn a_session_refuses_work_after_shutdown() {
    let mut guest = Guest::load(&module("morphir_avro_extension.wasm")).expect("the module loads");
    guest.initialize().expect("the session starts");
    guest
        .request(
            morphir_extension_sdk::protocol::methods::SHUTDOWN,
            json!({}),
        )
        .expect("shutdown answers");
    let refused = guest.generate(v4::v4_customer_library(), "avro", json!({}));
    assert!(refused.is_err(), "generate after shutdown: {refused:?}");
}
