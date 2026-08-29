//! Canonical JSON hashing for strict trajectory request and result structs.

use std::fmt;

use serde::{
    Serialize, Serializer,
    ser::{
        self, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
};
use serde_json::{Number, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;
use wellforge_trajectory_contract::TrajectoryAnalysisResult;

/// Failure to produce canonical finite JSON.
#[derive(Debug, Error)]
pub(crate) enum CanonicalError {
    /// A typed floating-point value was not finite.
    #[error("canonical JSON rejects non-finite floating-point values")]
    NonFinite,
    /// Strict JSON serialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug)]
struct ValidationError;

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("non-finite floating-point value")
    }
}

impl std::error::Error for ValidationError {}

impl ser::Error for ValidationError {
    fn custom<T: fmt::Display>(_message: T) -> Self {
        Self
    }
}

struct FiniteValidator;
struct FiniteCompound;

impl Serializer for FiniteValidator {
    type Ok = ();
    type Error = ValidationError;
    type SerializeSeq = FiniteCompound;
    type SerializeTuple = FiniteCompound;
    type SerializeTupleStruct = FiniteCompound;
    type SerializeTupleVariant = FiniteCompound;
    type SerializeMap = FiniteCompound;
    type SerializeStruct = FiniteCompound;
    type SerializeStructVariant = FiniteCompound;

    fn serialize_bool(self, _value: bool) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_i8(self, _value: i8) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_i16(self, _value: i16) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_i32(self, _value: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_i64(self, _value: i64) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_u8(self, _value: u8) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_u16(self, _value: u16) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_u32(self, _value: u32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_u64(self, _value: u64) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_f32(self, value: f32) -> Result<(), Self::Error> {
        if value.is_finite() {
            Ok(())
        } else {
            Err(ValidationError)
        }
    }

    fn serialize_f64(self, value: f64) -> Result<(), Self::Error> {
        if value.is_finite() {
            Ok(())
        } else {
            Err(ValidationError)
        }
    }

    fn serialize_char(self, _value: char) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_str(self, _value: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_none(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(self, _length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(FiniteCompound)
    }

    fn serialize_tuple(self, _length: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(FiniteCompound)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(FiniteCompound)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(FiniteCompound)
    }

    fn serialize_map(self, _length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(FiniteCompound)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(FiniteCompound)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(FiniteCompound)
    }
}

impl SerializeSeq for FiniteCompound {
    type Ok = ();
    type Error = ValidationError;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SerializeTuple for FiniteCompound {
    type Ok = ();
    type Error = ValidationError;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SerializeTupleStruct for FiniteCompound {
    type Ok = ();
    type Error = ValidationError;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SerializeTupleVariant for FiniteCompound {
    type Ok = ();
    type Error = ValidationError;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SerializeMap for FiniteCompound {
    type Ok = ();
    type Error = ValidationError;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> {
        key.serialize(FiniteValidator)
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SerializeStruct for FiniteCompound {
    type Ok = ();
    type Error = ValidationError;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl SerializeStructVariant for FiniteCompound {
    type Ok = ();
    type Error = ValidationError;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(FiniteValidator)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn normalize_negative_zero(value: &mut Value) {
    match value {
        Value::Array(values) => values.iter_mut().for_each(normalize_negative_zero),
        Value::Object(values) => values.values_mut().for_each(normalize_negative_zero),
        Value::Number(number) => {
            if number
                .as_f64()
                .is_some_and(|value| value == 0.0 && value.is_sign_negative())
            {
                *number = Number::from_f64(0.0).expect("zero is a finite JSON number");
            }
        }
        Value::Null | Value::Bool(_) | Value::String(_) => {}
    }
}

/// Serializes a strict value as compact canonical JSON after recursively normalizing negative zero.
pub(crate) fn bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalError> {
    value
        .serialize(FiniteValidator)
        .map_err(|_| CanonicalError::NonFinite)?;
    let mut value = serde_json::to_value(value)?;
    normalize_negative_zero(&mut value);
    Ok(serde_json::to_vec(&value)?)
}

/// Returns the lowercase hexadecimal SHA-256 digest of canonical JSON.
pub(crate) fn hash<T: Serialize>(value: &T) -> Result<String, CanonicalError> {
    Ok(hex::encode(Sha256::digest(bytes(value)?)))
}

/// Hashes a result after blanking only its result-hash evidence field.
pub(crate) fn result_hash(result: &TrajectoryAnalysisResult) -> Result<String, CanonicalError> {
    let mut normalized = result.clone();
    normalized.evidence.result_hash.clear();
    hash(&normalized)
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::hash;

    #[derive(Serialize)]
    struct OptionalNumber {
        value: Option<f64>,
    }

    #[test]
    fn canonical_hash_rejects_nonfinite_optional_floats_instead_of_hashing_null() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(hash(&OptionalNumber { value: Some(value) }).is_err());
        }
    }
}
