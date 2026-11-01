//! MEP `0.2.0-draft.1` model generation for the tuple `morphir-ir` /
//! `morphir-json` / envelope `0.1.0-draft.1` / IR `4.0.0`.
//!
//! `morphir.backend.generateModel` admits the envelope and runs the generation
//! of `morphir.backend.generate` on its IR, so both lines give the same
//! artifacts and diagnostics for the same IR. In a session, the SDK checks the
//! agreed tuple, the limits and the payload before dispatch; this module checks
//! the envelope again for a direct call.

use morphir_extension_sdk::model::{
    GenerateModelRequest, MODEL_ENVELOPE_VERSION, ModelCodec, ModelOperation, ModelTransportLimits,
    ModelTransportSupport, ModelTransportTuple, retain_model,
};
use morphir_extension_sdk::{ExtensionError, GenerateRequest, GenerateResult};

/// The model-content profile that the extension reads.
pub const PROFILE: &str = "morphir-ir";
/// The semantic IR version of the model payload.
pub const IR_VERSION: &str = "4.0.0";

/// The one tuple that the extension serves: JSON, because the SDK has no Ion
/// reader for a WASM guest.
pub fn tuple() -> ModelTransportTuple {
    ModelTransportTuple {
        operation: ModelOperation::GenerateModel,
        profile: PROFILE.into(),
        codec: ModelCodec::MorphirJson,
        envelope_version: MODEL_ENVELOPE_VERSION.into(),
        ir_version: IR_VERSION.into(),
    }
}

/// The model transport that the extension offers. A host takes the lower of
/// each limit, so these are the extension's own upper bounds.
pub fn support() -> ModelTransportSupport {
    ModelTransportSupport {
        tuples: vec![tuple()],
        limits: ModelTransportLimits {
            max_encoded_bytes: 64 * 1024 * 1024,
            max_decoded_bytes: 48 * 1024 * 1024,
            max_depth: 1024,
            max_nodes: 16 * 1024 * 1024,
        },
        guarantees: Vec::new(),
    }
}

/// Admit the envelope of a generation request and return its IR as a
/// `morphir.backend.generate` request.
///
/// An unagreed tuple, an envelope that does not match the tuple or its digest,
/// and a required guarantee are refused as invalid parameters: the extension
/// checks no guarantee.
pub fn generate_request(
    request: GenerateModelRequest,
) -> morphir_extension_sdk::Result<GenerateRequest> {
    let tuple = tuple();
    if request.input != tuple {
        return Err(ExtensionError::InvalidParams(
            "Unsupported model input tuple".into(),
        ));
    }
    if let Some(required) = request.required_guarantees.first() {
        return Err(ExtensionError::InvalidParams(format!(
            "The extension cannot check the required guarantee '{}'",
            required.guarantee
        )));
    }
    let retained = retain_model(&request.model, &tuple, &support().limits)
        .map_err(|error| ExtensionError::InvalidParams(error.to_string()))?;
    let ir = serde_json::from_slice(retained.bytes())
        .map_err(|error| ExtensionError::InvalidParams(format!("Model payload: {error}")))?;
    Ok(GenerateRequest {
        ir,
        target: request.target,
        options: request.options,
    })
}

/// Run `morphir.backend.generateModel` with the given generation function.
pub fn generate_model(
    request: GenerateModelRequest,
    generate: impl FnOnce(GenerateRequest) -> morphir_extension_sdk::Result<GenerateResult>,
) -> morphir_extension_sdk::Result<GenerateResult> {
    generate(generate_request(request)?)
}
