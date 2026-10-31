//! A test-only Extism host for the extension guests of this repository.
//!
//! [`Guest`] loads a built module with no WASI and no host functions beyond the
//! Extism kernel, and sends MEP JSON-RPC requests to its `handle` export. The
//! release workflow uses it to read the claims of the exact bytes it releases,
//! and the tests use it to compare the module with the native crate.

use std::path::Path;

use extism::{Manifest, Plugin, Wasm};
use morphir_extension_sdk::protocol::methods;
use serde_json::{Value, json};

/// The MEP version that the guests of this repository speak.
pub const PROTOCOL_VERSION: &str = "0.1";

/// An error from loading a guest or from one exchange with it.
#[derive(Debug)]
pub struct GuestError(pub String);

impl std::fmt::Display for GuestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for GuestError {}

fn error(message: impl Into<String>) -> GuestError {
    GuestError(message.into())
}

/// One loaded guest module.
pub struct Guest {
    plugin: Plugin,
    next_id: u64,
}

impl Guest {
    /// Load the module at `path` without WASI.
    pub fn load(path: &Path) -> Result<Self, GuestError> {
        let manifest = Manifest::new([Wasm::file(path)]);
        let plugin = Plugin::new(&manifest, [], false)
            .map_err(|e| error(format!("cannot load {}: {e}", path.display())))?;
        Ok(Self { plugin, next_id: 1 })
    }

    /// Send one request and return its `result`, or the JSON-RPC error as an error.
    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, GuestError> {
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        let input = serde_json::to_vec(&request).expect("a JSON value serializes");
        let output: Vec<u8> = self
            .plugin
            .call("handle", input)
            .map_err(|e| error(format!("{method}: the guest trapped: {e}")))?;
        let response: Value = serde_json::from_slice(&output)
            .map_err(|e| error(format!("{method}: the answer is not JSON: {e}")))?;
        if response.get("id") != Some(&json!(id)) {
            return Err(error(format!(
                "{method}: the answer has another id: {response}"
            )));
        }
        if let Some(rpc_error) = response.get("error").filter(|e| !e.is_null()) {
            return Err(error(format!("{method}: {rpc_error}")));
        }
        response
            .get("result")
            .cloned()
            .ok_or_else(|| error(format!("{method}: the answer has no result: {response}")))
    }

    /// Ask for the guest's capability claim set without starting a session.
    pub fn describe(&mut self) -> Result<Value, GuestError> {
        self.request(
            methods::DESCRIBE,
            json!({ "protocolVersions": [PROTOCOL_VERSION] }),
        )
    }

    /// Start a session with `morphir.initialize` and return its result.
    ///
    /// The `morphir.initialized` notification is not sent: the SDK's Extism
    /// guests answer only requests through `handle`, so Extism hosts drop
    /// notifications (as the Rust host does).
    pub fn initialize(&mut self) -> Result<Value, GuestError> {
        let result = self.request(
            methods::INITIALIZE,
            json!({
                "protocolVersions": [PROTOCOL_VERSION],
                "host": {
                    "kind": "cli",
                    "name": "extension-guest-check",
                    "version": env!("CARGO_PKG_VERSION"),
                },
            }),
        )?;
        let negotiated = result.get("protocolVersion").and_then(Value::as_str);
        if negotiated != Some(PROTOCOL_VERSION) {
            return Err(error(format!(
                "initialize negotiated {negotiated:?}: {result}"
            )));
        }
        Ok(result)
    }

    /// Send `morphir.backend.generate` in an open session.
    pub fn generate(
        &mut self,
        ir: Value,
        target: &str,
        options: Value,
    ) -> Result<Value, GuestError> {
        self.request(
            methods::GENERATE,
            json!({ "ir": ir, "target": target, "options": options }),
        )
    }

    /// End the session: `morphir.shutdown`, then `morphir.exit`.
    pub fn shutdown(&mut self) -> Result<(), GuestError> {
        self.request(methods::SHUTDOWN, json!({}))?;
        self.request(methods::EXIT, json!({}))?;
        Ok(())
    }
}
