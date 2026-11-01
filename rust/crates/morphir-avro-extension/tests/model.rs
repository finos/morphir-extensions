//! MEP `0.2.0-draft.1`: negotiation, `morphir.backend.generateModel` and its
//! refusals.
//!
//! The session tests drive the SDK guest session with the dispatcher that
//! `export_extension!` builds for the Extism `handle` export, so they run the
//! code path of the released module on the native target.

use std::collections::HashMap;
use std::path::PathBuf;

use morphir_avro_extension::AvroExtension;
use morphir_extension_sdk::model::{
    GenerateModelRequest, MODEL_ENVELOPE_VERSION, ModelCodec, ModelGuaranteeRequirement,
    ModelOperation, ModelPayload, ModelTransportTuple, MorphirModel,
};
use morphir_extension_sdk::protocol::{ExtensionRequest, error_codes, methods};
use morphir_extension_sdk::{
    __ProtocolSession, __dispatch_backend, __dispatch_request, Backend, ExtensionError,
    ExtensionType, GenerateRequest, GenerateResult,
};
use pretty_assertions::assert_eq;
use serde_json::{Value, json};

const MODEL_MEP: &str = "0.2.0-draft.1";
const CONTEXT_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn json_tuple() -> Value {
    json!({
        "operation": "morphir.backend.generateModel",
        "profile": "morphir-ir",
        "codec": "morphir-json",
        "envelopeVersion": "0.1.0-draft.1",
        "irVersion": "4.0.0",
    })
}

fn ion_tuple() -> Value {
    json!({
        "operation": "morphir.backend.generateModel",
        "profile": "morphir-model",
        "codec": "ion-binary",
        "envelopeVersion": "0.1.0-draft.1",
        "irVersion": "4.0.0",
    })
}

/// The limits that the MoonBit worker host offers.
fn host_limits() -> Value {
    json!({
        "maxEncodedBytes": 2_796_204,
        "maxDecodedBytes": 2_097_152,
        "maxDepth": 128,
        "maxNodes": 100_000,
    })
}

fn tuple() -> ModelTransportTuple {
    serde_json::from_value(json_tuple()).expect("the JSON tuple decodes")
}

/// One guest session, as the Extism `handle` export runs it.
struct Session {
    session: __ProtocolSession,
    next_id: u64,
}

impl Session {
    fn new() -> Self {
        Self {
            session: __ProtocolSession::new(),
            next_id: 1,
        }
    }

    /// Send one request; return its result, or its JSON-RPC error code.
    fn call(&mut self, method: &str, params: Value) -> Result<Value, i32> {
        let request = ExtensionRequest::new(method, params, self.next_id).expect("request");
        self.next_id += 1;
        let response = self.session.dispatch(&request, || {
            __dispatch_request::<AvroExtension>(
                &request,
                &[__dispatch_backend::<AvroExtension>],
                &[ExtensionType::Backend],
            )
        });
        match response.error {
            Some(error) => Err(error.code),
            None => Ok(response
                .result
                .expect("a response without an error has a result")),
        }
    }

    fn initialize(&mut self, versions: &[&str], model_transport: Option<Value>) -> Value {
        let mut params = json!({
            "protocolVersions": versions,
            "host": { "kind": "cli", "name": "model-test", "version": "0.0.0" },
        });
        if let Some(offer) = model_transport {
            params["modelTransport"] = offer;
        }
        self.call(methods::INITIALIZE, params)
            .expect("the session starts")
    }

    /// A session on the model line with the MoonBit CLI's default offer: Ion
    /// first, then JSON.
    fn model_line() -> Self {
        let mut session = Self::new();
        let result = session.initialize(
            &[MODEL_MEP, "0.1"],
            Some(json!({ "tuples": [ion_tuple(), json_tuple()], "limits": host_limits() })),
        );
        assert_eq!(result["protocolVersion"], MODEL_MEP);
        session
    }
}

fn canonical_customer() -> Value {
    serde_json::from_str(include_str!("fixtures/canonical-customer-v4.json"))
        .expect("the fixture is JSON")
}

fn envelope(ir: &Value) -> MorphirModel {
    let bytes = serde_json::to_vec(ir).expect("the IR serializes");
    let payload = ModelPayload::from_bytes(ModelCodec::MorphirJson, &bytes).expect("payload");
    MorphirModel {
        envelope_version: MODEL_ENVELOPE_VERSION.into(),
        model_id: "examples/customer".into(),
        revision: format!("sha256:{}", payload.sha256),
        ir_version: "4.0.0".into(),
        profile: "morphir-ir".into(),
        payload,
        evidence: Vec::new(),
        critical: Vec::new(),
    }
}

fn model_request(model: MorphirModel, target: &str, options: Value) -> GenerateModelRequest {
    GenerateModelRequest {
        model,
        input: tuple(),
        target: target.into(),
        context_digest: CONTEXT_DIGEST.into(),
        required_guarantees: Vec::new(),
        options: to_map(&options),
    }
}

fn to_map(options: &Value) -> HashMap<String, Value> {
    options
        .as_object()
        .expect("options are an object")
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn generate(ir: Value, target: &str, options: &Value) -> GenerateResult {
    AvroExtension
        .generate(GenerateRequest {
            ir,
            target: target.into(),
            options: to_map(options),
        })
        .expect("generation is a successful MEP call")
}

/// A reviewed model golden in `tests/model`. `UPDATE_GOLDEN=1` rewrites it,
/// except in CI.
fn golden(name: &str, actual: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/model")
        .join(name);
    if std::env::var_os("UPDATE_GOLDEN").is_some_and(|value| value == "1") {
        assert!(
            std::env::var_os("CI").is_none(),
            "refusing to update goldens in CI"
        );
        std::fs::write(&path, actual).expect("the golden file is writable");
    }
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing golden file {name}"))
}

// --- Claims ----------------------------------------------------------------

#[test]
fn claims_list_both_protocol_lines_and_the_json_tuple() {
    let mut session = Session::new();
    let claims = session
        .call(methods::DESCRIBE, json!({ "protocolVersions": ["0.1"] }))
        .expect("describe answers a 0.1 host");

    assert_eq!(claims["claimsVersion"], "0.1.0-draft.2");
    assert_eq!(claims["protocolVersions"], json!([MODEL_MEP, "0.1"]));
    assert_eq!(
        claims["capabilities"]["modelTransport"]["tuples"],
        json!([json_tuple()])
    );
    assert_eq!(
        claims["capabilities"]["modelTransport"]["guarantees"],
        json!([])
    );
    assert_eq!(claims["capabilities"]["backend"]["generate"], true);
}

// --- Negotiation -----------------------------------------------------------

#[test]
fn a_host_that_offers_only_0_1_gets_the_0_1_session() {
    let mut session = Session::new();
    let result = session.initialize(&["0.1"], None);

    assert_eq!(result["protocolVersion"], "0.1");
    assert!(
        result["capabilities"].get("modelTransport").is_none(),
        "{result:#}"
    );

    let generated = session
        .call(
            methods::GENERATE,
            json!({ "ir": canonical_customer(), "target": "avro", "options": {} }),
        )
        .expect("generate answers on 0.1");
    assert_eq!(
        generated,
        serde_json::to_value(generate(canonical_customer(), "avro", &json!({}))).unwrap()
    );

    let refused = session.call(
        methods::GENERATE_MODEL,
        serde_json::to_value(model_request(
            envelope(&canonical_customer()),
            "avro",
            json!({}),
        ))
        .unwrap(),
    );
    assert_eq!(refused, Err(error_codes::CAPABILITY_UNAVAILABLE));
}

#[test]
fn a_host_without_a_model_offer_gets_0_1() {
    let mut session = Session::new();
    let result = session.initialize(&[MODEL_MEP, "0.1"], None);

    assert_eq!(result["protocolVersion"], "0.1");
    assert!(
        result["capabilities"].get("modelTransport").is_none(),
        "{result:#}"
    );
}

#[test]
fn a_0_2_host_that_prefers_ion_agrees_on_json() {
    let mut session = Session::new();
    let result = session.initialize(
        &[MODEL_MEP, "0.1"],
        Some(json!({ "tuples": [ion_tuple(), json_tuple()], "limits": host_limits() })),
    );

    assert_eq!(result["protocolVersion"], MODEL_MEP);
    assert_eq!(
        result["capabilities"]["modelTransport"],
        json!({ "tuples": [json_tuple()], "limits": host_limits(), "guarantees": [] })
    );
}

#[test]
fn a_host_that_requires_ion_gets_an_empty_agreement() {
    let mut session = Session::new();
    let result = session.initialize(
        &[MODEL_MEP],
        Some(json!({ "tuples": [ion_tuple()], "limits": host_limits() })),
    );

    assert_eq!(result["protocolVersion"], MODEL_MEP);
    assert_eq!(
        result["capabilities"]["modelTransport"]["tuples"],
        json!([])
    );
    let refused = session.call(
        methods::GENERATE_MODEL,
        serde_json::to_value(model_request(
            envelope(&canonical_customer()),
            "avro",
            json!({}),
        ))
        .unwrap(),
    );
    assert_eq!(refused, Err(error_codes::CAPABILITY_UNAVAILABLE));
}

// --- generateModel ---------------------------------------------------------

#[test]
fn generate_model_matches_generate_and_the_model_goldens() {
    let cases = [
        (
            json!({}),
            "examples/customer/domain/Customer.avsc",
            "canonical-customer.avsc",
        ),
        (
            json!({ "representation": "idl" }),
            "examples/customer/domain/CustomerSchemas.avdl",
            "canonical-customer.avdl",
        ),
    ];
    let mut session = Session::model_line();
    for (options, path, name) in cases {
        let result = session
            .call(
                methods::GENERATE_MODEL,
                serde_json::to_value(model_request(
                    envelope(&canonical_customer()),
                    "avro",
                    options.clone(),
                ))
                .unwrap(),
            )
            .unwrap_or_else(|code| panic!("{options}: error {code}"));

        assert_eq!(
            result,
            serde_json::to_value(generate(canonical_customer(), "avro", &options)).unwrap(),
            "{options}"
        );
        assert_eq!(result["success"], true);
        let artifacts = result["artifacts"].as_array().expect("artifacts");
        assert_eq!(artifacts.len(), 1, "{options}: {artifacts:?}");
        assert_eq!(artifacts[0]["path"], path);
        let content = artifacts[0]["content"].as_str().expect("text content");
        assert_eq!(content, golden(name, content), "{name}");
    }
}

#[test]
fn generate_model_keeps_backend_failures_as_results() {
    let mut session = Session::model_line();
    let result = session
        .call(
            methods::GENERATE_MODEL,
            serde_json::to_value(model_request(
                envelope(&canonical_customer()),
                "avro",
                json!({ "representation": "xml" }),
            ))
            .unwrap(),
        )
        .expect("an invalid option is a backend result, not a protocol error");

    assert_eq!(result["success"], false);
    assert_eq!(result["artifacts"], json!([]));
    assert_eq!(result["diagnostics"][0]["code"], "AVRO004");
}

// --- Refusals --------------------------------------------------------------

fn refused_in_session(request: GenerateModelRequest) -> Result<Value, i32> {
    Session::model_line().call(
        methods::GENERATE_MODEL,
        serde_json::to_value(request).unwrap(),
    )
}

fn assert_refused(request: GenerateModelRequest, label: &str) {
    assert_eq!(
        refused_in_session(request.clone()),
        Err(error_codes::INVALID_PARAMS),
        "{label} in a session"
    );
    match AvroExtension.generate_model(request) {
        Err(ExtensionError::InvalidParams(_)) => {}
        other => panic!("{label} in a direct call: {other:?}"),
    }
}

#[test]
fn an_unagreed_tuple_is_refused() {
    let mut request = model_request(envelope(&canonical_customer()), "avro", json!({}));
    request.input.ir_version = "3.0.0".into();
    request.model.ir_version = "3.0.0".into();
    assert_refused(request, "IR 3.0.0 tuple");

    let mut request = model_request(envelope(&canonical_customer()), "avro", json!({}));
    request.input.operation = ModelOperation::CompileModel;
    assert_refused(request, "compileModel tuple");
}

#[test]
fn a_wrong_envelope_is_refused() {
    let mut request = model_request(envelope(&canonical_customer()), "avro", json!({}));
    request.model.envelope_version = "0.2.0".into();
    assert_refused(request, "envelope version 0.2.0");

    let mut request = model_request(envelope(&canonical_customer()), "avro", json!({}));
    request.model.payload.sha256 = "1".repeat(64);
    assert_refused(request, "wrong digest");

    let mut request = model_request(envelope(&canonical_customer()), "avro", json!({}));
    request.model.profile = "morphir-model".into();
    assert_refused(request, "profile that differs from the tuple");

    let mut v3 = canonical_customer();
    v3["formatVersion"] = json!(3);
    assert_refused(
        model_request(envelope(&v3), "avro", json!({})),
        "payload formatVersion that differs from the tuple",
    );
}

#[test]
fn a_required_guarantee_is_refused() {
    let mut request = model_request(envelope(&canonical_customer()), "avro", json!({}));
    request.required_guarantees = vec![ModelGuaranteeRequirement {
        guarantee: "morphir.typed".into(),
        scope: "examples/customer".into(),
    }];
    assert_refused(request, "required guarantee");
}
