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
//!
//! The same module owns the enum flatten (t88, the f120/f121 class's
//! generation-side lever): adopted-repo agents emit enum values
//! unquoted — `{"status": open}` — while plain string fields are
//! always quoted, and the values sat two indirections away behind
//! `$ref → $defs oneOf-of-consts`. Every vocabulary enum is
//! `#[schemars(inline)]` (dto.rs, edges.rs), so its oneOf-of-consts
//! reaches every use site inline, and [`flatten_inline_enums`] rewrites
//! it to the flat idiom the function-calling training distribution
//! uses — `{"type": "string", "enum": ["open", …]}`, the values quoted
//! in declaration order. Outputs get the flatten inside
//! [`output_for_type`]; inputs get it through [`flatten_input`] over
//! the router's advertised schemas (lib.rs), because the `#[tool]`
//! macro generates input schemas inside rmcp with no seam of ours.
//! Struct definitions stay `$ref`s — the failure evidence is
//! enum-specific — and the wire pin in tests/read/query.rs walks both
//! sides of every advertisement.

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
    flatten_inline_enums(Arc::make_mut(&mut schema));
    schema
}

/// The input side of the enum flatten: the one seam this crate owns
/// over an advertised input schema. The `#[tool]` macro generates
/// input schemas inside rmcp (`schema_for_input`, thread-locally
/// cached), so the router walk in `TodoServer::new` is where they can
/// be post-processed — this function is its per-tool step, called once
/// per tool at startup. The `Arc::make_mut` clone leaves rmcp's cache
/// entry untouched, the same isolation `output_for_type` relies on.
pub(crate) fn flatten_input(schema: &mut Arc<JsonObject>) {
    flatten_inline_enums(Arc::make_mut(schema));
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

// =================================================================
// The enum flatten (t88)
// =================================================================

/// Rewrites every inlined vocabulary-enum schema in `map` from
/// schemars' oneOf-of-consts spelling to the flat idiom: the `oneOf`
/// key becomes `type: "string"` plus `enum: [values]`, the values in
/// arm order — the enum's declaration order, the same order the
/// derive emits. The object's other keys (a property's field
/// description) stay; the per-variant descriptions drop with the arms,
/// and the enum's own type-level description drops with the array
/// element it rode in on — the field-level descriptions carry the
/// meaning, the values are the point. An object whose `oneOf` is not
/// all-const-arms (a tagged union like `BlockerDto`, a struct) is
/// untouched.
fn flatten_inline_enums(map: &mut Map<String, Value>) {
    let values = match map.get("oneOf") {
        Some(Value::Array(arms)) => const_values(arms),
        _ => None,
    };
    if let Some(values) = values {
        map.remove("oneOf");
        insert_flat_keys(map, values);
    }
    for value in map.values_mut() {
        flatten_value(value);
    }
}

/// The recursive half of the flatten: a schema is a JSON value, and
/// every object or array inside it may carry a oneOf-of-consts. An
/// array element that is exactly an inlined enum schema — at most a
/// `description` beside the oneOf, the arm `Option<Enum>` builds — is
/// replaced by the bare flat form; anything else is walked, so a
/// property's own keys survive beside its flattened pair.
fn flatten_value(value: &mut Value) {
    match value {
        Value::Object(map) => flatten_inline_enums(map),
        Value::Array(items) => {
            for item in items.iter_mut() {
                if let Some(flat) = bare_flat(item) {
                    *item = flat;
                } else {
                    flatten_value(item);
                }
            }
        }
        _ => {}
    }
}

/// The flat idiom's values when `arms` is schemars' oneOf-of-consts
/// spelling of a string enum — every arm exactly a string `const` with
/// an optional per-variant description, in arm order. `None` when any
/// arm is anything else, so an unrecognized shape survives untouched
/// for the wire pin to catch rather than being silently rewritten.
fn const_values(arms: &[Value]) -> Option<Vec<Value>> {
    if arms.is_empty() {
        return None;
    }
    let mut values = Vec::with_capacity(arms.len());
    for arm in arms {
        let map = arm.as_object()?;
        let is_const_arm = map.get("type").and_then(Value::as_str) == Some("string")
            && map.get("const").is_some_and(Value::is_string)
            && map
                .keys()
                .all(|key| key == "description" || key == "type" || key == "const");
        if !is_const_arm {
            return None;
        }
        values.push(map.get("const").cloned()?);
    }
    Some(values)
}

/// The bare flat form for an array element that is exactly an inlined
/// enum schema — at most a `description` beside the oneOf-of-consts.
/// `None` for anything else, which falls through to the walk and is
/// flattened in place, keeping its other keys.
fn bare_flat(item: &Value) -> Option<Value> {
    let map = item.as_object()?;
    let is_bare =
        map.contains_key("oneOf") && map.keys().all(|key| key == "description" || key == "oneOf");
    if !is_bare {
        return None;
    }
    let arms = match map.get("oneOf") {
        Some(Value::Array(arms)) => arms,
        _ => return None,
    };
    let values = const_values(arms)?;
    let mut flat = Map::new();
    insert_flat_keys(&mut flat, values);
    Some(Value::Object(flat))
}

/// Inserts the flat idiom's two keys — `type: "string"` then the
/// `enum` values — into `map`, in that order.
fn insert_flat_keys(map: &mut Map<String, Value>, values: Vec<Value>) {
    map.insert("type".to_owned(), Value::String("string".to_owned()));
    map.insert("enum".to_owned(), Value::Array(values));
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

    /// A non-nullable enum property — the field description beside the
    /// inlined oneOf-of-consts — flattens to the field description
    /// beside the flat pair, the values in arm order.
    #[test]
    fn a_non_nullable_enum_property_flattens_in_place() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "kind": {
                    "description": "What kind of work this records.",
                    "oneOf": [
                        { "description": "One unit.", "type": "string", "const": "task" },
                        { "description": "A checkpoint.", "type": "string", "const": "gate" }
                    ]
                }
            }
        });
        flatten_value(&mut schema);
        assert_eq!(
            schema,
            json!({
                "type": "object",
                "properties": {
                    "kind": {
                        "description": "What kind of work this records.",
                        "type": "string",
                        "enum": ["task", "gate"]
                    }
                }
            })
        );
    }

    /// A nullable enum property — the `anyOf` arm carrying the enum's
    /// own type-level description beside the oneOf-of-consts — keeps
    /// the `anyOf` wrapper with the flattened arm bare and the null
    /// arm last: no type-array, no `null` inside the values.
    #[test]
    fn a_nullable_enum_property_keeps_its_anyof_wrapper() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "status": {
                    "description": "A new lifecycle status.",
                    "anyOf": [
                        {
                            "description": "The wire mirror of the status.",
                            "oneOf": [
                                { "description": "Not started.", "type": "string", "const": "open" },
                                { "type": "string", "const": "in_work" }
                            ]
                        },
                        { "type": "null" }
                    ]
                }
            }
        });
        flatten_value(&mut schema);
        assert_eq!(
            schema,
            json!({
                "type": "object",
                "properties": {
                    "status": {
                        "description": "A new lifecycle status.",
                        "anyOf": [
                            { "type": "string", "enum": ["open", "in_work"] },
                            { "type": "null" }
                        ]
                    }
                }
            })
        );
    }

    /// A `oneOf` whose arms are not all string consts — a tagged
    /// union like `BlockerDto`, or a mixed combinator — is not an
    /// enum vocabulary and survives untouched.
    #[test]
    fn a_non_const_oneof_survives() {
        let mut schema = json!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "shape": { "const": "task" },
                        "status": { "type": "string", "enum": ["open"] }
                    }
                },
                { "type": "string", "const": "external" }
            ]
        });
        let before = schema.clone();
        flatten_value(&mut schema);
        assert_eq!(schema, before);
    }

    /// An empty `oneOf` is not a vocabulary (the flat idiom's values
    /// are non-empty by the wire pin) and survives untouched.
    #[test]
    fn an_empty_oneof_survives() {
        let mut schema = json!({ "oneOf": [] });
        let before = schema.clone();
        flatten_value(&mut schema);
        assert_eq!(schema, before);
    }

    /// An array element carrying keys beyond `description` beside its
    /// oneOf-of-consts is not the bare inlined-enum shape: it falls
    /// through to the walk and flattens in place, keeping its keys.
    #[test]
    fn an_element_with_extra_keys_flattens_in_place() {
        let mut schema = json!({
            "anyOf": [
                {
                    "description": "The type-level doc.",
                    "default": "open",
                    "oneOf": [
                        { "type": "string", "const": "open" },
                        { "type": "string", "const": "closed" }
                    ]
                },
                { "type": "null" }
            ]
        });
        flatten_value(&mut schema);
        assert_eq!(
            schema,
            json!({
                "anyOf": [
                    {
                        "description": "The type-level doc.",
                        "default": "open",
                        "type": "string",
                        "enum": ["open", "closed"]
                    },
                    { "type": "null" }
                ]
            })
        );
    }
}
