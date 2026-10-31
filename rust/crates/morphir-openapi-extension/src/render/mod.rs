//! Renderers that turn a dialect-neutral [`crate::SchemaProjection`] into a
//! dialect-specific document.
//!
//! [`json_schema`] renders JSON Schema 2020-12 documents and [`openapi`]
//! renders an OpenAPI 3.1 document. Both share the schema-body conversion
//! defined in this module, differing only in the base a `$ref` is written
//! against.

mod downgrade;
pub mod json_schema;
pub mod openapi;

pub use json_schema::render_json_schema;
pub use openapi::render_openapi;

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::{DecimalDigits, DecimalEncoding, NamedSchema, Schema, SchemaVariant};

/// Orders every object's members by name, at every depth.
///
/// Both renderers assemble their document as a [`Value`] and print it, so without this the
/// bytes they write would follow the order the builder happened to insert members in. Member
/// order carries no meaning in JSON Schema or OpenAPI, so sorting makes a rendered artifact a
/// function of the projection alone and keeps a reviewed document reviewable by diff.
pub(crate) fn in_canonical_member_order(value: Value) -> Value {
    match value {
        Value::Object(members) => Value::Object(
            members
                .into_iter()
                .map(|(name, member)| (name, in_canonical_member_order(member)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(in_canonical_member_order).collect())
        }
        scalar => scalar,
    }
}

/// The JSON object body of one named schema: its own keywords, plus the
/// `x-morphir-fqname` and optional `description` every named schema carries.
///
/// Shared by both renderers: `reference_base` is the only place
/// reference-base knowledge lives, so the JSON Schema renderer (`#/$defs/`)
/// and the OpenAPI renderer (`#/components/schemas/`) produce the same body
/// for the same [`NamedSchema`], apart from where a `$ref` points.
pub(crate) fn named_schema_body(named: &NamedSchema, reference_base: &str) -> Map<String, Value> {
    let mut object = as_object(schema_body(&named.schema, reference_base));
    object.insert("x-morphir-fqname".to_owned(), json!(named.source_name));
    if let Some(doc) = &named.doc {
        object.insert("description".to_owned(), json!(doc));
    }
    object
}

/// The schema keywords for one [`Schema`], with every `$ref` written against
/// `reference_base`.
///
/// This is the shared schema-body conversion: `reference_base` is the only
/// place reference-base knowledge lives, so the JSON Schema renderer
/// (`#/$defs/`) and the OpenAPI renderer (`#/components/schemas/`) reuse it
/// unchanged, apart from where a `$ref` points.
pub(crate) fn schema_body(schema: &Schema, reference_base: &str) -> Value {
    let mut object = Map::new();
    match schema {
        Schema::Boolean => {
            object.insert("type".to_owned(), json!("boolean"));
        }
        Schema::Integer { format } => {
            object.insert("type".to_owned(), json!("integer"));
            if let Some(format) = format {
                object.insert("format".to_owned(), json!(format));
            }
        }
        Schema::Number { format } => {
            object.insert("type".to_owned(), json!("number"));
            if let Some(format) = format {
                object.insert("format".to_owned(), json!(format));
            }
        }
        Schema::Decimal { encoding, digits } => {
            // The registered OpenAPI format `decimal` applies to both JSON
            // types, so a generator can still choose a decimal type.
            object.insert("format".to_owned(), json!("decimal"));
            match encoding {
                DecimalEncoding::String => {
                    object.insert("type".to_owned(), json!("string"));
                    object.insert("pattern".to_owned(), json!(decimal_pattern(*digits)));
                }
                DecimalEncoding::Number => {
                    object.insert("type".to_owned(), json!("number"));
                }
            }
        }
        Schema::Text { max_length } => {
            object.insert("type".to_owned(), json!("string"));
            if let Some(max_length) = max_length {
                object.insert("maxLength".to_owned(), json!(max_length));
            }
        }
        Schema::Null => {
            object.insert("type".to_owned(), json!("null"));
        }
        Schema::Array { items, unique } => {
            object.insert("type".to_owned(), json!("array"));
            object.insert("items".to_owned(), schema_body(items, reference_base));
            if *unique {
                object.insert("uniqueItems".to_owned(), json!(true));
            }
        }
        Schema::Tuple(members) => {
            let prefix_items: Vec<Value> = members
                .iter()
                .map(|member| schema_body(member, reference_base))
                .collect();
            let count = prefix_items.len();
            object.insert("type".to_owned(), json!("array"));
            object.insert("prefixItems".to_owned(), Value::Array(prefix_items));
            object.insert("items".to_owned(), json!(false));
            object.insert("minItems".to_owned(), json!(count));
            object.insert("maxItems".to_owned(), json!(count));
        }
        Schema::Map { values } => {
            object.insert("type".to_owned(), json!("object"));
            object.insert(
                "additionalProperties".to_owned(),
                schema_body(values, reference_base),
            );
        }
        Schema::Object { fields, required } => {
            object.insert("type".to_owned(), json!("object"));
            let mut properties = Map::new();
            for field in fields {
                let mut property = as_object(schema_body(&field.schema, reference_base));
                if let Some(doc) = &field.doc {
                    property.insert("description".to_owned(), json!(doc));
                }
                properties.insert(field.name.clone(), Value::Object(property));
            }
            object.insert("properties".to_owned(), Value::Object(properties));
            if !required.is_empty() {
                object.insert("required".to_owned(), json!(required));
            }
        }
        Schema::Enumeration(values) => {
            object.insert("type".to_owned(), json!("string"));
            object.insert("enum".to_owned(), json!(values));
        }
        Schema::OneOf {
            discriminator,
            variants,
        } => {
            let variants = variants
                .iter()
                .map(|variant| Value::Object(variant_body(variant, discriminator, reference_base)))
                .collect();
            object.insert("oneOf".to_owned(), Value::Array(variants));
        }
        // Always `anyOf`, even when every member is a simple type: a member
        // can be a `Schema::Reference`, and `anyOf` stays correct for that
        // case, so there is no separate "all members are simple" special case.
        Schema::Union(members) => {
            let members = members
                .iter()
                .map(|member| schema_body(member, reference_base))
                .collect();
            object.insert("anyOf".to_owned(), Value::Array(members));
        }
        Schema::Reference(name) => {
            object.insert("$ref".to_owned(), json!(format!("{reference_base}{name}")));
        }
    }
    Value::Object(object)
}

/// The anchored pattern of a `Decimal` string.
///
/// Unbounded, it is a positional profile: plain decimal notation with an
/// optional minus sign, the form every Morphir data writer emits (`0`,
/// `-0.05`, `33500`). It is not a canonical spelling, so it also accepts
/// `-0` and trailing fraction zeros (`10.50`). It refuses an exponent, a `+`
/// sign, `.5`, `1.` and leading zeros, which no Morphir data writer emits;
/// several of them do not parse in morphir-elm's data decoder, which then
/// reads 0 without an error. With [`DecimalDigits`], the integer part has at
/// most `precision - scale` digits and the fraction at most `scale` digits.
/// The pattern uses only syntax that ECMA-262 and the Rust `regex` crate
/// read alike.
fn decimal_pattern(digits: Option<DecimalDigits>) -> String {
    let Some(DecimalDigits { precision, scale }) = digits else {
        return r"^-?(0|[1-9][0-9]*)(\.[0-9]+)?$".to_owned();
    };
    let integer = match precision.saturating_sub(scale) {
        0 => "0".to_owned(),
        integer_digits => format!("(0|[1-9][0-9]{{0,{}}})", integer_digits - 1),
    };
    let fraction = match scale {
        0 => String::new(),
        scale => format!(r"(\.[0-9]{{1,{scale}}})?"),
    };
    format!("^-?{integer}{fraction}$")
}

/// One [`Schema::OneOf`] variant, with its discriminator property fixed to
/// the constructor name by `const`.
///
/// The insert below is unconditional, and safe to be: the projection rejects
/// a constructor argument that would project to the discriminator's own
/// property name (`JSC003`, in `schema::types::reject_discriminator_collision`),
/// so no variant reaching here already declares it. Were that check removed,
/// this would overwrite the real property and push a duplicate into
/// `required` — invalid under JSON Schema 2020-12 and the OpenAPI 3.0
/// metaschema alike.
fn variant_body(
    variant: &SchemaVariant,
    discriminator: &str,
    reference_base: &str,
) -> Map<String, Value> {
    let mut object = as_object(schema_body(&variant.schema, reference_base));
    if let Some(Value::Object(properties)) = object.get_mut("properties") {
        properties.insert(discriminator.to_owned(), json!({ "const": variant.name }));
    }
    let mut required: Vec<Value> = object
        .get("required")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    required.push(json!(discriminator));
    object.insert("required".to_owned(), Value::Array(required));
    object
}

/// Unwrap a [`schema_body`] result back into its object, for a caller that
/// needs to add more keys before serializing.
///
/// `schema_body` always returns `Value::Object`: every [`Schema`] variant
/// builds a JSON object, never a bare array or scalar.
fn as_object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(object) => object,
        other => unreachable!("schema_body always returns an object, got {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DecimalDigits, DecimalEncoding};

    fn decimal(encoding: DecimalEncoding, digits: Option<(u32, u32)>) -> Value {
        schema_body(
            &Schema::Decimal {
                encoding,
                digits: digits.map(|(precision, scale)| DecimalDigits { precision, scale }),
            },
            "#/$defs/",
        )
    }

    /// Validate each instance against `schema` and return those it accepts.
    fn accepted<'a>(schema: &Value, instances: &[&'a Value]) -> Vec<&'a Value> {
        let validator = jsonschema::validator_for(schema).expect("a valid schema");
        instances
            .iter()
            .copied()
            .filter(|instance| validator.is_valid(instance))
            .collect()
    }

    #[test]
    fn renders_the_default_decimal_as_a_pattern_checked_string() {
        assert_eq!(
            decimal(DecimalEncoding::String, None),
            json!({
                "type": "string",
                "format": "decimal",
                "pattern": "^-?(0|[1-9][0-9]*)(\\.[0-9]+)?$"
            })
        );
    }

    #[test]
    fn the_default_decimal_accepts_exactly_the_canonical_strings() {
        let valid = [
            json!("0"),
            json!("-0.05"),
            json!("33500"),
            json!("12.34"),
            json!("10.50"),
        ];
        let invalid = [
            json!("1e3"),
            json!("+1"),
            json!(".5"),
            json!("1."),
            json!("007"),
            json!(""),
            json!(" 1"),
            json!(12.5),
        ];
        let schema = decimal(DecimalEncoding::String, None);

        assert_eq!(
            accepted(&schema, &valid.iter().collect::<Vec<_>>()),
            valid.iter().collect::<Vec<_>>()
        );
        assert!(accepted(&schema, &invalid.iter().collect::<Vec<_>>()).is_empty());
    }

    #[test]
    fn renders_the_number_encoding_as_a_decimal_number() {
        assert_eq!(
            decimal(DecimalEncoding::Number, None),
            json!({ "type": "number", "format": "decimal" })
        );
    }

    #[test]
    fn bounds_the_integer_and_fraction_digits() {
        let schema = decimal(DecimalEncoding::String, Some((10, 2)));

        assert_eq!(
            schema["pattern"],
            json!("^-?(0|[1-9][0-9]{0,7})(\\.[0-9]{1,2})?$")
        );
        let valid = [json!("12345678.90"), json!("-0.5"), json!("0")];
        let invalid = [json!("123456789.0"), json!("1.234")];
        assert_eq!(
            accepted(&schema, &valid.iter().collect::<Vec<_>>()),
            valid.iter().collect::<Vec<_>>()
        );
        assert!(accepted(&schema, &invalid.iter().collect::<Vec<_>>()).is_empty());
    }

    #[test]
    fn a_zero_scale_allows_no_fraction() {
        let schema = decimal(DecimalEncoding::String, Some((5, 0)));

        assert_eq!(schema["pattern"], json!("^-?(0|[1-9][0-9]{0,4})$"));
        let valid = [json!("12345"), json!("-7")];
        let invalid = [json!("123456"), json!("1.5")];
        assert_eq!(
            accepted(&schema, &valid.iter().collect::<Vec<_>>()),
            valid.iter().collect::<Vec<_>>()
        );
        assert!(accepted(&schema, &invalid.iter().collect::<Vec<_>>()).is_empty());
    }

    #[test]
    fn a_scale_equal_to_the_precision_allows_only_a_zero_integer_part() {
        let schema = decimal(DecimalEncoding::String, Some((3, 3)));

        assert_eq!(schema["pattern"], json!("^-?0(\\.[0-9]{1,3})?$"));
        let valid = [json!("0.123"), json!("-0.5"), json!("0")];
        let invalid = [json!("1.5"), json!("0.1234")];
        assert_eq!(
            accepted(&schema, &valid.iter().collect::<Vec<_>>()),
            valid.iter().collect::<Vec<_>>()
        );
        assert!(accepted(&schema, &invalid.iter().collect::<Vec<_>>()).is_empty());
    }

    /// The largest precision still compiles in the Rust `regex` engine that
    /// the validator uses.
    #[test]
    fn the_largest_precision_compiles() {
        let schema = decimal(
            DecimalEncoding::String,
            Some((crate::MAX_DECIMAL_PRECISION, 10)),
        );

        let validator = jsonschema::validator_for(&schema).expect("a valid schema");
        assert!(validator.is_valid(&json!("12345678901234567890.0123456789")));
    }
}
