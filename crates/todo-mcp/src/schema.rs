//! The advertised schemas' format discipline (board U32, resolved by
//! the user's in-thread direction 2026-10-07): schemars emits OpenAPI
//! integer formats (`"uint64"` for `u64`, …) that JSON Schema does not
//! standardize, so a client-side validator warns on every `tools/list`
//! — opencode's Ajv printed `unknown format "uint64" ignored` lines
//! against this server at the t39 smoke (f99) — and a strict-mode
//! client could reject the schemas outright. Every schema this server
//! advertises therefore strips the integer-family format hints:
//! [`output_for_type`] wraps rmcp's `schema_for_type` at every
//! `output_schema` site, and [`optional_u64`] is the `schema_with`
//! spelling for the one `u64` input field (`next_items`'s `limit`).
//! The strip is surgical — exactly the integer family, never a
//! standard format like `date-time` — and pinned from the wire by the
//! `tools/list` test that walks every advertised schema
//! (tests/read/query.rs), which is also what closes the class: a
//! future `u64` field that bypasses these two seams fails that test.

use std::any::Any;
use std::sync::Arc;

use rmcp::model::JsonObject;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde_json::{Map, Value};

/// The OpenAPI integer-family formats schemars emits for the primitive
/// integer types — the ones JSON Schema validators do not know. The
/// family, not the instance: only `"uint64"` exists on this server
/// today, but every primitive integer spelling schemars emits —
/// `int8`…`int128`, `uint8`…`uint128`, and the `isize`/`usize`
/// spellings `int` and `uint` — is stripped, so the discipline does
/// not depend on which types happen to be in the DTOs. The number
/// formats (`float`, `double`) are out of scope: §8 bars
/// floating-point from anything the wire carries.
const INTEGER_FORMATS: [&str; 12] = [
    "int8", "int16", "int32", "int64", "int128", "int", "uint8", "uint16", "uint32", "uint64",
    "uint128", "uint",
];

/// The output schema for `T` as this server advertises it: rmcp's
/// `schema_for_type` (draft 2020-12, declaration-order properties under
/// D14's `preserve_order`) with the integer-family format hints
/// stripped. The call-site spelling at every `output_schema` in the
/// cluster files.
///
/// rmcp caches the generated schema behind an `Arc` shared with its
/// thread-local cache; `Arc::make_mut` clones the map into this call's
/// own allocation and leaves the cache's entry untouched, so the strip
/// never mutates a schema another caller might hold. The router is
/// built once per process, so the clone is paid once per type at
/// startup, never per request.
pub(crate) fn output_for_type<T: JsonSchema + Any>() -> Arc<JsonObject> {
    let mut schema = rmcp::handler::server::tool::schema_for_type::<T>();
    strip_integer_formats(Arc::make_mut(&mut schema));
    schema
}

/// The schema for an optional count field (`Option<u64>`), spelled
/// without the format hint: `["integer", "null"]` with `minimum: 0`.
/// Delegates to schemars' own `Option<u64>` implementation — so the
/// nullability semantics are schemars', never hand-written — and then
/// strips the integer-family format it emits. The `schema_with` target
/// for `next_items`'s `limit`, the one `u64` input field; reusable by
/// any future optional-count parameter.
pub(crate) fn optional_u64(generator: &mut SchemaGenerator) -> Schema {
    let mut schema = <Option<u64> as JsonSchema>::json_schema(generator);
    if let Some(object) = schema.as_object_mut() {
        strip_integer_formats(object);
    }
    schema
}

/// Removes `"format"` keys carrying an integer-family value from every
/// depth of `map` — objects, arrays and `$defs` included, because the
/// DTO schemas nest their sub-schemas under `$defs` with `$ref`
/// pointers. A standard format (`date-time`, `uri`, …) survives, and a
/// non-string `"format"` value survives: the family check is what
/// decides, not the key's presence.
fn strip_integer_formats(map: &mut Map<String, Value>) {
    let is_integer_format = matches!(
        map.get("format"),
        Some(Value::String(format)) if INTEGER_FORMATS.contains(&format.as_str())
    );
    if is_integer_format {
        map.remove("format");
    }
    for value in map.values_mut() {
        strip_value(value);
    }
}

/// The recursive half of the strip: a schema is a JSON value, and every
/// object or array inside it may carry a `"format"` key.
fn strip_value(value: &mut Value) {
    match value {
        Value::Object(map) => strip_integer_formats(map),
        Value::Array(items) => {
            for item in items.iter_mut() {
                strip_value(item);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    /// The walk reaches every depth: a `$defs`-shaped nest with an
    /// array between the root and the offender is stripped, the
    /// surrounding structure untouched.
    #[test]
    fn nested_formats_are_stripped_at_every_depth() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": { "type": "object", "properties": {
                        "weight": { "type": "integer", "format": "uint64", "minimum": 0 }
                    }}
                }
            },
            "$defs": {
                "DeferredDto": {
                    "type": "object",
                    "properties": {
                        "total": { "type": "integer", "format": "uint64", "minimum": 0 },
                        "ready": { "type": "integer", "format": "uint64", "minimum": 0 }
                    }
                }
            }
        });
        strip_value(&mut schema);
        assert_eq!(
            schema,
            json!({
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "items": { "type": "object", "properties": {
                            "weight": { "type": "integer", "minimum": 0 }
                        }}
                    }
                },
                "$defs": {
                    "DeferredDto": {
                        "type": "object",
                        "properties": {
                            "total": { "type": "integer", "minimum": 0 },
                            "ready": { "type": "integer", "minimum": 0 }
                        }
                    }
                }
            })
        );
    }

    /// The strip is the integer family, not the key: a standard format
    /// a validator knows survives it.
    #[test]
    fn standard_formats_survive() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "until": { "type": "string", "format": "date-time" },
                "count": { "type": "integer", "format": "uint64", "minimum": 0 }
            }
        });
        strip_value(&mut schema);
        assert_eq!(
            schema,
            json!({
                "type": "object",
                "properties": {
                    "until": { "type": "string", "format": "date-time" },
                    "count": { "type": "integer", "minimum": 0 }
                }
            })
        );
    }

    /// The family check is string-valued: a non-string `"format"`
    /// value is not an integer-family hint and survives untouched.
    #[test]
    fn non_string_format_values_survive() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "odd": { "type": "string", "format": 5 }
            }
        });
        strip_value(&mut schema);
        assert_eq!(
            schema,
            json!({
                "type": "object",
                "properties": {
                    "odd": { "type": "string", "format": 5 }
                }
            })
        );
    }
}
