//! A deterministic test-only serde codec, so the serde contract
//! (round-trip equality, twice-run byte identity, wire spellings, field
//! order) is testable before the canonical serializer (`dacode-json`)
//! lands with the store's golden harness — the only approved dependency
//! of this crate is `serde`, which ships no data format.
//!
//! Tokens: strings are length-prefixed (`9:camera:U7` — no escaping to
//! get wrong), integers `i42e`, `None` is `z`, sequences `l…e`, structs
//! `d…e` with length-prefixed field names in declaration order; enums
//! serialize externally tagged — a unit variant as its (renamed) name
//! string, a struct variant as that string followed by `d…e`. That
//! covers exactly the node schema and the crate's error enums; anything
//! else is an error, not a guess. These bytes are a test artifact with
//! no stability promise — the golden files of spec §10 pin the real
//! format.

use std::fmt;

use serde::de::value::StringDeserializer;
use serde::de::{self, DeserializeOwned, Deserializer, SeqAccess, Visitor};
use serde::forward_to_deserialize_any;
use serde::ser::{self, Serialize, SerializeSeq, SerializeStruct, Serializer};

const TAG_INT: u8 = b'i';
const TAG_NONE: u8 = b'z';
const TAG_SEQ: u8 = b'l';
const TAG_STRUCT: u8 = b'd';
const TAG_END: u8 = b'e';
/// Cap on length-prefixed token sizes, so a corrupt length cannot
/// ask for an absurd allocation.
const MAX_TOKEN_LEN: usize = 1 << 20;

/// The codec's error: a plain message, test-internal.
#[derive(Debug)]
pub(crate) struct CodecError(String);

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CodecError {}

impl ser::Error for CodecError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

impl de::Error for CodecError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

fn unsupported<T>(what: &str) -> Result<T, CodecError> {
    Err(CodecError(format!(
        "testcodec: {what} is outside the node schema"
    )))
}

/// Serializes `value` to the codec's canonical bytes.
pub(crate) fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CodecError> {
    let mut encoder = Encoder { out: Vec::new() };
    value.serialize(&mut encoder)?;
    Ok(encoder.out)
}

/// Deserializes a value from codec bytes; trailing bytes are an
/// error.
pub(crate) fn from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    let mut decoder = Decoder {
        input: bytes,
        pos: 0,
    };
    let value = T::deserialize(&mut decoder)?;
    if decoder.pos != bytes.len() {
        return Err(CodecError("testcodec: trailing bytes".to_owned()));
    }
    Ok(value)
}

struct Encoder {
    out: Vec<u8>,
}

impl Encoder {
    fn push_str(&mut self, s: &str) {
        self.out.extend_from_slice(s.len().to_string().as_bytes());
        self.out.push(b':');
        self.out.extend_from_slice(s.as_bytes());
    }
}

impl Serializer for &mut Encoder {
    type Ok = ();
    type Error = CodecError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<(), CodecError> {
        unsupported("bool")
    }
    fn serialize_i8(self, _v: i8) -> Result<(), CodecError> {
        unsupported("i8")
    }
    fn serialize_i16(self, _v: i16) -> Result<(), CodecError> {
        unsupported("i16")
    }
    fn serialize_i32(self, _v: i32) -> Result<(), CodecError> {
        unsupported("i32")
    }
    fn serialize_i64(self, _v: i64) -> Result<(), CodecError> {
        unsupported("i64")
    }
    fn serialize_u8(self, _v: u8) -> Result<(), CodecError> {
        unsupported("u8")
    }
    fn serialize_u16(self, _v: u16) -> Result<(), CodecError> {
        unsupported("u16")
    }
    fn serialize_u32(self, _v: u32) -> Result<(), CodecError> {
        unsupported("u32")
    }
    fn serialize_u64(self, v: u64) -> Result<(), CodecError> {
        self.out.push(TAG_INT);
        self.out.extend_from_slice(v.to_string().as_bytes());
        self.out.push(TAG_END);
        Ok(())
    }
    fn serialize_f32(self, _v: f32) -> Result<(), CodecError> {
        unsupported("floats (§8: no floats in persisted state)")
    }
    fn serialize_f64(self, _v: f64) -> Result<(), CodecError> {
        unsupported("floats (§8: no floats in persisted state)")
    }
    fn serialize_char(self, _v: char) -> Result<(), CodecError> {
        unsupported("char")
    }
    fn serialize_str(self, v: &str) -> Result<(), CodecError> {
        self.push_str(v);
        Ok(())
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<(), CodecError> {
        unsupported("bytes")
    }
    fn serialize_none(self) -> Result<(), CodecError> {
        self.out.push(TAG_NONE);
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, v: &T) -> Result<(), CodecError> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), CodecError> {
        unsupported("unit")
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), CodecError> {
        unsupported("unit structs")
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<(), CodecError> {
        self.push_str(variant);
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        v: &T,
    ) -> Result<(), CodecError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<(), CodecError> {
        unsupported("newtype variants")
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, CodecError> {
        self.out.push(TAG_SEQ);
        Ok(self)
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, CodecError> {
        unsupported("tuples")
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, CodecError> {
        unsupported("tuple structs")
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, CodecError> {
        unsupported("tuple variants")
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, CodecError> {
        unsupported("maps (the schema uses sorted sets and structs)")
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, CodecError> {
        self.out.push(TAG_STRUCT);
        Ok(self)
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, CodecError> {
        self.push_str(variant);
        self.out.push(TAG_STRUCT);
        Ok(self)
    }
}

impl SerializeSeq for &mut Encoder {
    type Ok = ();
    type Error = CodecError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<(), CodecError> {
        v.serialize(&mut **self)
    }
    fn end(self) -> Result<(), CodecError> {
        self.out.push(TAG_END);
        Ok(())
    }
}

impl SerializeStruct for &mut Encoder {
    type Ok = ();
    type Error = CodecError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        v: &T,
    ) -> Result<(), CodecError> {
        self.push_str(key);
        v.serialize(&mut **self)
    }
    fn end(self) -> Result<(), CodecError> {
        self.out.push(TAG_END);
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Encoder {
    type Ok = ();
    type Error = CodecError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _v: &T) -> Result<(), CodecError> {
        unsupported("tuples")
    }
    fn end(self) -> Result<(), CodecError> {
        unsupported("tuples")
    }
}

impl ser::SerializeTupleStruct for &mut Encoder {
    type Ok = ();
    type Error = CodecError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _v: &T) -> Result<(), CodecError> {
        unsupported("tuple structs")
    }
    fn end(self) -> Result<(), CodecError> {
        unsupported("tuple structs")
    }
}

impl ser::SerializeTupleVariant for &mut Encoder {
    type Ok = ();
    type Error = CodecError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _v: &T) -> Result<(), CodecError> {
        unsupported("tuple variants")
    }
    fn end(self) -> Result<(), CodecError> {
        unsupported("tuple variants")
    }
}

impl ser::SerializeMap for &mut Encoder {
    type Ok = ();
    type Error = CodecError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, _k: &T) -> Result<(), CodecError> {
        unsupported("maps")
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, _v: &T) -> Result<(), CodecError> {
        unsupported("maps")
    }
    fn end(self) -> Result<(), CodecError> {
        unsupported("maps")
    }
}

impl ser::SerializeStructVariant for &mut Encoder {
    type Ok = ();
    type Error = CodecError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        v: &T,
    ) -> Result<(), CodecError> {
        self.push_str(key);
        v.serialize(&mut **self)
    }
    fn end(self) -> Result<(), CodecError> {
        self.out.push(TAG_END);
        Ok(())
    }
}

struct Decoder<'i> {
    input: &'i [u8],
    pos: usize,
}

impl Decoder<'_> {
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek();
        if byte.is_some() {
            self.pos += 1;
        }
        byte
    }

    fn expect(&mut self, tag: u8) -> Result<(), CodecError> {
        if self.bump() == Some(tag) {
            Ok(())
        } else {
            Err(CodecError(format!(
                "testcodec: expected '{}' at offset {}",
                char::from(tag),
                self.pos
            )))
        }
    }

    fn parse_str(&mut self) -> Result<String, CodecError> {
        let start = self.pos;
        let mut len = 0usize;
        while let Some(b) = self.peek() {
            if !b.is_ascii_digit() {
                break;
            }
            len = len.saturating_mul(10).saturating_add(usize::from(b - b'0'));
            self.pos += 1;
        }
        if self.pos == start {
            return Err(CodecError(format!(
                "testcodec: expected a length-prefixed string at offset {start}"
            )));
        }
        if self.bump() != Some(b':') || len > MAX_TOKEN_LEN {
            return Err(CodecError(format!(
                "testcodec: malformed string token at offset {start}"
            )));
        }
        let end = self.pos + len;
        let slice = self
            .input
            .get(self.pos..end)
            .ok_or_else(|| CodecError("testcodec: truncated string".to_owned()))?;
        let value = String::from_utf8(slice.to_vec())
            .map_err(|_| CodecError("testcodec: string is not UTF-8".to_owned()))?;
        self.pos = end;
        Ok(value)
    }

    fn parse_u64(&mut self) -> Result<u64, CodecError> {
        self.expect(TAG_INT)?;
        let start = self.pos;
        while matches!(self.peek(), Some(b) if b.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(CodecError("testcodec: malformed integer".to_owned()));
        }
        let digits = self
            .input
            .get(start..self.pos)
            .and_then(|raw| std::str::from_utf8(raw).ok())
            .ok_or_else(|| CodecError("testcodec: malformed integer".to_owned()))?;
        let value = digits
            .parse::<u64>()
            .map_err(|_| CodecError("testcodec: integer out of range".to_owned()))?;
        self.expect(TAG_END)?;
        Ok(value)
    }
}

impl<'de, 'a, 'i> Deserializer<'de> for &'a mut Decoder<'i> {
    type Error = CodecError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CodecError> {
        match self.peek() {
            Some(b) if b.is_ascii_digit() => visitor.visit_string(self.parse_str()?),
            Some(TAG_INT) => visitor.visit_u64(self.parse_u64()?),
            _ => unsupported("`any` over none/seq/struct tokens"),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CodecError> {
        visitor.visit_string(self.parse_str()?)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CodecError> {
        visitor.visit_string(self.parse_str()?)
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CodecError> {
        visitor.visit_u64(self.parse_u64()?)
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CodecError> {
        if self.peek() == Some(TAG_NONE) {
            self.pos += 1;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CodecError> {
        self.expect(TAG_SEQ)?;
        visitor.visit_seq(SeqAcc { de: self })
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, CodecError> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, CodecError> {
        self.expect(TAG_STRUCT)?;
        // The derive reads exactly the declared field count and
        // never asks for the terminating None, so the terminator is
        // consumed here, not inside the accessor.
        let value = visitor.visit_seq(StructAcc { de: &mut *self })?;
        self.expect(TAG_END)?;
        Ok(value)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, CodecError> {
        let variant = self.parse_str()?;
        visitor.visit_enum(EnumAcc { variant, de: self })
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CodecError> {
        visitor.visit_string(self.parse_str()?)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, CodecError> {
        self.deserialize_any(de::IgnoredAny)?;
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 f32 f64 char bytes byte_buf unit
        unit_struct tuple tuple_struct map
    }
}

struct SeqAcc<'d, 'i> {
    de: &'d mut Decoder<'i>,
}

impl<'de, 'd, 'i> SeqAccess<'de> for SeqAcc<'d, 'i> {
    type Error = CodecError;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, CodecError> {
        if self.de.peek() == Some(TAG_END) {
            self.de.pos += 1;
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct StructAcc<'d, 'i> {
    de: &'d mut Decoder<'i>,
}

impl<'de, 'd, 'i> SeqAccess<'de> for StructAcc<'d, 'i> {
    type Error = CodecError;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, CodecError> {
        // Field names were written in declaration order by the same
        // derive that reads them back; the value follows the name.
        let _name = self.de.parse_str()?;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct EnumAcc<'d, 'i> {
    variant: String,
    de: &'d mut Decoder<'i>,
}

impl<'de, 'd, 'i> de::EnumAccess<'de> for EnumAcc<'d, 'i> {
    type Error = CodecError;
    type Variant = VariantAcc<'d, 'i>;

    fn variant_seed<V: de::DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, VariantAcc<'d, 'i>), CodecError> {
        let de = StringDeserializer::<CodecError>::new(self.variant);
        let value = seed.deserialize(de)?;
        Ok((value, VariantAcc { de: self.de }))
    }
}

struct VariantAcc<'d, 'i> {
    de: &'d mut Decoder<'i>,
}

impl<'de, 'd, 'i> de::VariantAccess<'de> for VariantAcc<'d, 'i> {
    type Error = CodecError;

    fn unit_variant(self) -> Result<(), CodecError> {
        Ok(())
    }
    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(
        self,
        _seed: T,
    ) -> Result<T::Value, CodecError> {
        unsupported("newtype variants")
    }
    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, CodecError> {
        unsupported("tuple variants")
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, CodecError> {
        self.de.expect(TAG_STRUCT)?;
        let value = visitor.visit_seq(StructAcc { de: &mut *self.de })?;
        self.de.expect(TAG_END)?;
        Ok(value)
    }
}
