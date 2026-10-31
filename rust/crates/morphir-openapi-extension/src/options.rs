use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use serde_json::Value;

use crate::SchemaDiagnostic;

/// Configuration accepted by the OpenAPI and JSON Schema backend.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SchemaOptions {
    /// Unsupported-form handling policy.
    pub unsupported: Unsupported,
    /// OpenAPI document version to render.
    pub version: OpenApiVersion,
    /// Public-model surface projected into the OpenAPI document.
    pub projection: Projection,
    /// How a value operation's result is shaped into responses.
    pub result_responses: ResultResponses,
    /// HTTP status code used for the error response.
    pub error_status: u16,
    /// Per-operation overrides, keyed by canonical Morphir FQName.
    pub operations: BTreeMap<String, OperationOverride>,
    /// The JSON type that a Morphir SDK `Decimal` value is written as.
    pub decimal_encoding: DecimalEncoding,
    /// Total significant digits of a `Decimal`, as in the Avro extension.
    /// Unbounded when unset; set it together with [`Self::decimal_scale`].
    pub decimal_precision: Option<u32>,
    /// Digits after the decimal point of a `Decimal`, as in the Avro
    /// extension. Unbounded when unset; set it together with
    /// [`Self::decimal_precision`]. Trailing fraction zeros count, so
    /// `"1.500"` exceeds a scale of 2.
    pub decimal_scale: Option<u32>,
}

impl Default for SchemaOptions {
    fn default() -> Self {
        Self {
            unsupported: Unsupported::Error,
            version: OpenApiVersion::V31,
            projection: Projection::Schemas,
            result_responses: ResultResponses::Data,
            error_status: 400,
            operations: BTreeMap::new(),
            decimal_encoding: DecimalEncoding::String,
            decimal_precision: None,
            decimal_scale: None,
        }
    }
}

/// The largest `decimal_precision`: it becomes a counted repetition in the
/// `Decimal` pattern, and RE2 and the engines modelled on it (Go's `regexp`)
/// refuse a count above 1000.
pub const MAX_DECIMAL_PRECISION: u32 = 1000;

impl SchemaOptions {
    /// Decode backend options without coercing the JSON values supplied by the host.
    pub fn from_map(options: &HashMap<String, Value>) -> Result<Self, SchemaDiagnostic> {
        let options = options
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();

        let value = serde_json::to_value(options)
            .map_err(|error| SchemaDiagnostic::invalid_option(error.to_string()))?;
        let decoded: Self = serde_json::from_value(value)
            .map_err(|error| SchemaDiagnostic::invalid_option(error.to_string()))?;
        decoded.validate()?;
        Ok(decoded)
    }

    /// Validate ranges and shapes after decoding or direct construction.
    ///
    /// Projection entry points must call this when their options did not come
    /// from [`Self::from_map`].
    pub fn validate(&self) -> Result<(), SchemaDiagnostic> {
        if !(400..=599).contains(&self.error_status) {
            return Err(SchemaDiagnostic::invalid_option(format!(
                "error_status ({}) must be in the range 400 through 599",
                self.error_status
            )));
        }
        self.validate_decimal()?;
        for (source_name, operation) in &self.operations {
            if let Some(path) = &operation.path
                && !path.starts_with('/')
            {
                return Err(SchemaDiagnostic::invalid_option(format!(
                    "operation path for {source_name} must start with '/': {path}"
                )));
            }
        }
        Ok(())
    }

    /// The digit bounds of a `Decimal`, when both bounds are set.
    pub fn decimal_digits(&self) -> Option<DecimalDigits> {
        match (self.decimal_precision, self.decimal_scale) {
            (Some(precision), Some(scale)) => Some(DecimalDigits { precision, scale }),
            _ => None,
        }
    }

    /// Precision and scale come as a pair, bound the string encoding only,
    /// and keep the Avro extension's ranges: a precision of at least one and
    /// a scale no larger than the precision.
    fn validate_decimal(&self) -> Result<(), SchemaDiagnostic> {
        let (precision, scale) = match (self.decimal_precision, self.decimal_scale) {
            (None, None) => return Ok(()),
            (Some(precision), Some(scale)) => (precision, scale),
            _ => {
                return Err(SchemaDiagnostic::invalid_option(
                    "decimal_precision and decimal_scale must be set together",
                ));
            }
        };
        if self.decimal_encoding == DecimalEncoding::Number {
            return Err(SchemaDiagnostic::invalid_option(
                "decimal_precision and decimal_scale bound only decimal_encoding \"string\"; \
                 a JSON number has no exact digit bound",
            ));
        }
        if !(1..=MAX_DECIMAL_PRECISION).contains(&precision) {
            return Err(SchemaDiagnostic::invalid_option(format!(
                "decimal_precision ({precision}) must be in the range 1 through {MAX_DECIMAL_PRECISION}"
            )));
        }
        if scale > precision {
            return Err(SchemaDiagnostic::invalid_option(format!(
                "decimal_scale ({scale}) must not exceed decimal_precision ({precision})"
            )));
        }
        Ok(())
    }
}

/// The JSON type that a Morphir SDK `Decimal` value is written as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecimalEncoding {
    /// A string in plain decimal notation, which keeps the exact value.
    #[default]
    String,
    /// A JSON number, which most consumers read as binary floating point.
    Number,
}

/// The digit bounds of a `Decimal`: at most `precision` significant digits,
/// of which at most `scale` follow the decimal point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecimalDigits {
    /// Total significant digits.
    pub precision: u32,
    /// Digits after the decimal point.
    pub scale: u32,
}

/// How the backend reacts to a Morphir form it cannot project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Unsupported {
    /// Fail the whole generation and emit no artifacts.
    #[default]
    Error,
    /// Skip the form, warn at its Morphir FQName, and keep valid artifacts.
    WarnAndSkip,
}

/// The OpenAPI document version to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum OpenApiVersion {
    /// OpenAPI 3.1.
    #[default]
    #[serde(rename = "3.1")]
    V31,
    /// OpenAPI 3.0.
    #[serde(rename = "3.0")]
    V30,
}

/// The subset of the Morphir package projected into the OpenAPI document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Projection {
    /// Project public type roots and schemas only.
    #[default]
    Schemas,
    /// Project schemas plus declared application entry points as operations.
    OperationsEntryPoints,
    /// Project schemas plus all public value specifications as operations.
    OperationsPublic,
}

/// How a value operation's result is shaped into HTTP responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResultResponses {
    /// A single success response carrying the whole result value.
    #[default]
    Data,
    /// Separate success and error responses split from the result value.
    Split,
}

/// A per-operation override, keyed by canonical Morphir FQName.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OperationOverride {
    /// HTTP method override.
    pub method: Option<HttpMethod>,
    /// Path template override; must start with `/` when present.
    pub path: Option<String>,
    /// Parameter binding overrides, keyed by parameter name.
    pub parameters: BTreeMap<String, ParameterBinding>,
}

/// An HTTP method usable in an operation override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    /// HTTP GET.
    Get,
    /// HTTP PUT.
    Put,
    /// HTTP POST.
    Post,
    /// HTTP DELETE.
    Delete,
    /// HTTP PATCH.
    Patch,
}

/// Where an operation parameter is bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterBinding {
    /// Bound to a path segment.
    Path,
    /// Bound to a query parameter.
    Query,
    /// Bound to an HTTP header.
    Header,
    /// Bound to the request body.
    Body,
}
