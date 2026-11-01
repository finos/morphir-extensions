//! The released module on the model line, MEP `0.2.0-draft.1`.
//!
//! These tests need the release modules; see `parity.rs`. They negotiate as the
//! MoonBit worker host does (Ion first, then JSON), send
//! `morphir.backend.generateModel` with a JSON model envelope and compare the
//! answer with the native `morphir.backend.generate` of the same IR. The
//! `0.1` session of `parity.rs` stays the check of an older host.

use std::collections::HashMap;
use std::path::PathBuf;

use extension_guest_check::{Guest, MODEL_PROTOCOL_VERSION, PROTOCOL_VERSION};
use morphir_avro_extension::AvroExtension;
use morphir_extension_sdk::model::{MODEL_ENVELOPE_VERSION, ModelCodec, ModelPayload};
use morphir_extension_sdk::protocol::{error_codes, methods};
use morphir_extension_sdk::{Backend, GenerateRequest};
use morphir_openapi_extension::OpenApiExtension;
use morphir_projection::testing::v4;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};

const CONTEXT_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

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

fn json_tuple() -> Value {
    json!({
        "operation": "morphir.backend.generateModel",
        "profile": "morphir-ir",
        "codec": "morphir-json",
        "envelopeVersion": "0.1.0-draft.1",
        "irVersion": "4.0.0",
    })
}

/// The MoonBit worker host's default offer: Ion first, then JSON.
fn moonbit_offer() -> Value {
    json!({
        "tuples": [
            {
                "operation": "morphir.backend.generateModel",
                "profile": "morphir-model",
                "codec": "ion-binary",
                "envelopeVersion": "0.1.0-draft.1",
                "irVersion": "4.0.0",
            },
            json_tuple(),
        ],
        "limits": {
            "maxEncodedBytes": 2_796_204,
            "maxDecodedBytes": 2_097_152,
            "maxDepth": 128,
            "maxNodes": 100_000,
        },
    })
}

fn model_session(file: &str) -> Guest {
    let mut guest = Guest::load(&module(file)).expect("the module loads");
    let result = guest
        .initialize_with(
            &[MODEL_PROTOCOL_VERSION, PROTOCOL_VERSION],
            Some(moonbit_offer()),
            MODEL_PROTOCOL_VERSION,
        )
        .expect("the session starts on the model line");
    assert_eq!(
        result["capabilities"]["modelTransport"]["tuples"],
        json!([json_tuple()]),
        "{file} agrees on the JSON tuple only"
    );
    guest
}

fn fixture(path: &str) -> Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    serde_json::from_str(&std::fs::read_to_string(root.join(path)).expect("the fixture exists"))
        .expect("the fixture is JSON")
}

/// Every IR v4 input of the parity test.
fn models() -> Vec<(&'static str, Value)> {
    vec![
        ("v4 customer application", v4::v4_customer_application()),
        ("v4 customer library", v4::v4_customer_library()),
        ("v4 customer specs", v4::v4_customer_specs()),
        ("v4 incomplete library", v4::v4_incomplete_library()),
        (
            "canonical customer v4",
            fixture("morphir-avro-extension/tests/fixtures/canonical-customer-v4.json"),
        ),
        (
            "typed pricing v4",
            fixture("morphir-openapi-extension/tests/fixtures/typed-pricing.json"),
        ),
    ]
}

fn envelope(name: &str, ir: &Value) -> Value {
    let bytes = serde_json::to_vec(ir).expect("the IR serializes");
    let payload = ModelPayload::from_bytes(ModelCodec::MorphirJson, &bytes).expect("payload");
    json!({
        "envelopeVersion": MODEL_ENVELOPE_VERSION,
        "modelId": name,
        "revision": format!("sha256:{}", payload.sha256),
        "irVersion": "4.0.0",
        "profile": "morphir-ir",
        "payload": payload,
    })
}

fn model_request(name: &str, ir: &Value, target: &str, options: &Value) -> Value {
    json!({
        "model": envelope(name, ir),
        "input": json_tuple(),
        "target": target,
        "contextDigest": CONTEXT_DIGEST,
        "options": options,
    })
}

fn to_map(options: &Value) -> HashMap<String, Value> {
    options
        .as_object()
        .expect("options are an object")
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Every IR v4 model with every option set: `generateModel` on the module
/// equals the native `generate` of the same IR.
fn assert_model_parity<B: Backend>(
    native: &B,
    file: &str,
    targets: &[&str],
    option_sets: &[Value],
) {
    let mut guest = model_session(file);
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
                    .map(|result| serde_json::to_value(result).expect("the result serializes"))
                    .expect("the native generation is a successful MEP call");
                let response = guest
                    .generate_model(model_request(name, &ir, target, options))
                    .expect("the module answers");
                assert_eq!(
                    response["result"], expected,
                    "{file} {target} {options} {name}: {response}"
                );
                if expected["success"] == json!(true) {
                    generated += 1;
                }
            }
        }
    }
    guest.shutdown().expect("the session ends");
    assert!(generated > 0, "no case generated artifacts");
}

#[test]
#[ignore = "requires the release modules"]
fn avro_module_generates_from_a_model_envelope_as_the_native_crate() {
    assert_model_parity(
        &AvroExtension,
        "morphir_avro_extension.wasm",
        &["avro"],
        &[
            json!({}),
            json!({ "representation": "idl" }),
            json!({ "representation": "json", "projection": "protocol-entry-points" }),
            json!({ "unsupported": "warn-and-skip" }),
            json!({ "typo": true }),
        ],
    );
}

#[test]
#[ignore = "requires the release modules"]
fn openapi_module_generates_from_a_model_envelope_as_the_native_crate() {
    assert_model_parity(
        &OpenApiExtension,
        "morphir_openapi_extension.wasm",
        &["openapi", "json-schema"],
        &[
            json!({}),
            json!({ "version": "3.0" }),
            json!({ "projection": "operations-public" }),
            json!({ "unsupported": "warn-and-skip" }),
            json!({ "decimal_encoding": "number" }),
            json!({ "version": "2.0" }),
        ],
    );
}

const MODULES: [(&str, &str); 2] = [
    ("morphir_avro_extension.wasm", "avro"),
    ("morphir_openapi_extension.wasm", "openapi"),
];

#[test]
#[ignore = "requires the release modules"]
fn a_0_1_session_refuses_generate_model() {
    for (file, target) in MODULES {
        let mut guest = Guest::load(&module(file)).expect("the module loads");
        let result = guest.initialize().expect("the 0.1 session starts");
        assert!(
            result["capabilities"].get("modelTransport").is_none(),
            "{file}: {result}"
        );
        let response = guest
            .generate_model(model_request(
                "customer",
                &v4::v4_customer_library(),
                target,
                &json!({}),
            ))
            .expect("the module answers");
        assert_eq!(
            response["error"]["code"],
            error_codes::CAPABILITY_UNAVAILABLE,
            "{file}: {response}"
        );
        guest.shutdown().expect("the session ends");
    }
}

#[test]
#[ignore = "requires the release modules"]
fn describe_claims_both_lines_for_either_offer() {
    for (file, _) in MODULES {
        let mut guest = Guest::load(&module(file)).expect("the module loads");
        let old = guest.describe().expect("describe answers a 0.1 host");
        let new = guest
            .request(
                methods::DESCRIBE,
                json!({ "protocolVersions": [MODEL_PROTOCOL_VERSION, PROTOCOL_VERSION] }),
            )
            .expect("describe answers a 0.2 host");
        assert_eq!(old, new, "{file}");
        assert_eq!(
            old["protocolVersions"],
            json!([MODEL_PROTOCOL_VERSION, PROTOCOL_VERSION]),
            "{file}"
        );
    }
}

#[test]
#[ignore = "requires the release modules"]
fn the_module_refuses_unagreed_tuples_and_wrong_envelopes() {
    let ir = fixture("morphir-avro-extension/tests/fixtures/canonical-customer-v4.json");
    for (file, target) in MODULES {
        let base = model_request("customer", &ir, target, &json!({}));
        let mut cases: Vec<(&str, Value)> = Vec::new();

        let mut request = base.clone();
        request["input"]["irVersion"] = json!("3.0.0");
        request["model"]["irVersion"] = json!("3.0.0");
        cases.push(("IR 3.0.0 tuple", request));

        let mut request = base.clone();
        request["model"]["envelopeVersion"] = json!("0.2.0");
        cases.push(("envelope version 0.2.0", request));

        let mut request = base.clone();
        request["model"]["payload"]["sha256"] = json!("1".repeat(64));
        cases.push(("wrong digest", request));

        let mut request = base.clone();
        request["requiredGuarantees"] =
            json!([{ "guarantee": "morphir.typed", "scope": "customer" }]);
        cases.push(("required guarantee", request));

        let mut guest = model_session(file);
        for (label, request) in cases {
            let response = guest.generate_model(request).expect("the module answers");
            assert_eq!(
                response["error"]["code"],
                error_codes::INVALID_PARAMS,
                "{file} {label}: {response}"
            );
            assert!(response.get("result").is_none(), "{file} {label}");
        }
        // The session still serves an agreed request after the refusals.
        let response = guest.generate_model(base).expect("the module answers");
        assert_eq!(response["result"]["success"], true, "{file}: {response}");
        guest.shutdown().expect("the session ends");
    }
}
