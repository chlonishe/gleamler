use std::io::Write;

use crate::serde::{atoms, error::Error};
use crate::wrapper::list::make_list;
use crate::{Encoder, Env, OwnedBinary, Term, types::tuple};
use serde::ser::{self, Serialize};

#[inline]
pub fn to_term<T>(env: Env, value: T) -> Result<Term, Error>
where
    T: Serialize,
{
    value.serialize(Serializer::from(env))
}

#[inline]
pub fn to_term_with<T>(env: Env, value: T, non_finite_float_as_atom: bool) -> Result<Term, Error>
where
    T: Serialize,
{
    value.serialize(Serializer::from(env).with_non_finite_float_as_atom(non_finite_float_as_atom))
}

#[derive(Clone, Copy)]
pub struct Serializer<'a> {
    env: Env<'a>,
    non_finite_float_as_atom: bool,
}

impl<'a> From<Env<'a>> for Serializer<'a> {
    fn from(env: Env<'a>) -> Serializer<'a> {
        Serializer {
            env,
            non_finite_float_as_atom: false,
        }
    }
}

impl<'a> Serializer<'a> {
    pub fn with_non_finite_float_as_atom(mut self, enabled: bool) -> Self {
        self.non_finite_float_as_atom = enabled;
        self
    }
}

impl<'a> ser::Serializer for Serializer<'a> {
    type Ok = Term<'a>;
    type Error = Error;

    type SerializeSeq = SequenceSerializer<'a>;
    type SerializeTuple = SequenceSerializer<'a>;
    type SerializeTupleStruct = SequenceSerializer<'a>;
    type SerializeTupleVariant = SequenceSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = MapSerializer<'a>;
    type SerializeStructVariant = MapSerializer<'a>;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(atoms::none().encode(self.env))
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let inner = value.serialize(self)?;
        Ok((atoms::some().encode(self.env), inner).encode(self.env))
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        if !v.is_finite() {
            return if self.non_finite_float_as_atom {
                if v.is_nan() {
                    Ok(atoms::nan().encode(self.env))
                } else if v.is_sign_positive() {
                    Ok(atoms::inf().encode(self.env))
                } else {
                    Ok(atoms::neg_inf().encode(self.env))
                }
            } else {
                Err(Error::NonFiniteFloat)
            };
        }
        Ok(v.encode(self.env))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(v.encode(self.env))
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let env = self.env;
        let str_len = v.len();
        let mut bin = OwnedBinary::new(str_len)
            .ok_or_else(|| Error::SerializationError("binary term allocation failed".into()))?;
        bin.as_mut_slice()
            .write_all(v.as_bytes())
            .map_err(|e| Error::SerializationError(format!("memory copy failed: {e}")))?;
        Ok(bin.release(env).to_term(env))
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut binary = OwnedBinary::new(v.len())
            .ok_or_else(|| Error::SerializationError("binary term allocation failed".into()))?;
        binary
            .as_mut_slice()
            .write_all(v)
            .map_err(|e| Error::SerializationError(format!("memory copy failed: {e}")))?;
        Ok(binary.release(self.env).to_term(self.env))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(().encode(self.env))
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(atoms::str_to_term(self.env, variant)?)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let name_term = atoms::str_to_term(self.env, name)?;
        let mut ser = SequenceSerializer::new(self, Some(2), Some(name_term));
        ser.add(value.serialize(self)?);
        ser.to_tuple()
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let effective_name = if name == "Result" || name.ends_with("::Result") {
            "Result"
        } else {
            name
        };
        match (effective_name, variant) {
            ("Result", "Ok") => self.serialize_newtype_struct("ok", value),
            ("Result", "Err") => self.serialize_newtype_struct("error", value),
            _ => self.serialize_newtype_struct(variant, value),
        }
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SequenceSerializer::new(self, len, None))
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SequenceSerializer::new(self, Some(len), None))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        let name_term = atoms::str_to_term(self.env, name)?;
        Ok(SequenceSerializer::new(self, Some(len), Some(name_term)))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_tuple_struct(variant, len)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(MapSerializer::new(self, len))
    }

    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(MapSerializer::new(self, Some(len)))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let name_term = atoms::str_to_term(self.env, variant)?;
        Ok(MapSerializer::new(self, Some(len + 1)).with_struct_variant_name(name_term))
    }
}

/// SequenceSerializer
pub struct SequenceSerializer<'a> {
    ser: Serializer<'a>,
    items: Vec<Term<'a>>,
}

impl<'a> SequenceSerializer<'a> {
    #[inline]
    fn new(ser: Serializer<'a>, len: Option<usize>, name: Option<Term<'a>>) -> Self {
        let mut items = match len {
            None => Vec::new(),
            Some(length) => Vec::with_capacity(length),
        };
        if let Some(name_term) = name {
            items.push(name_term);
        }
        SequenceSerializer { ser, items }
    }

    #[inline]
    fn add(&mut self, term: Term<'a>) {
        self.items.push(term)
    }

    #[inline]
    fn to_list(&self) -> Result<Term<'a>, Error> {
        let env = self.ser.env;
        let term_array: Vec<_> = self
            .items
            .iter()
            .map(|x| x.encode(env).as_c_arg())
            .collect();
        unsafe { Ok(Term::new(env, make_list(env.as_c_arg(), &term_array))) }
    }

    #[inline]
    fn to_tuple(&self) -> Result<Term<'a>, Error> {
        Ok(tuple::make_tuple(self.ser.env, &self.items))
    }
}

impl<'a> ser::SerializeSeq for SequenceSerializer<'a> {
    type Ok = Term<'a>;
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.add(value.serialize(self.ser)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Term<'a>, Error> {
        self.to_list()
    }
}

impl<'a> ser::SerializeTuple for SequenceSerializer<'a> {
    type Ok = Term<'a>;
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.add(value.serialize(self.ser)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Term<'a>, Error> {
        self.to_tuple()
    }
}

impl<'a> ser::SerializeTupleStruct for SequenceSerializer<'a> {
    type Ok = Term<'a>;
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Term<'a>, Error> {
        ser::SerializeTuple::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for SequenceSerializer<'a> {
    type Ok = Term<'a>;
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeTuple::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<Term<'a>, Error> {
        ser::SerializeTuple::end(self)
    }
}

/// MapSerializer
pub struct MapSerializer<'a> {
    ser: Serializer<'a>,
    keys: Vec<Term<'a>>,
    values: Vec<Term<'a>>,
    struct_variant_name: Option<Term<'a>>,
}

impl<'a> MapSerializer<'a> {
    #[inline]
    fn new(ser: Serializer<'a>, len: Option<usize>) -> Self {
        match len {
            None => MapSerializer {
                ser,
                keys: Vec::new(),
                values: Vec::new(),
                struct_variant_name: None,
            },
            Some(length) => MapSerializer {
                ser,
                keys: Vec::with_capacity(length),
                values: Vec::with_capacity(length),
                struct_variant_name: None,
            },
        }
    }

    #[inline]
    fn with_struct_variant_name(mut self, name: Term<'a>) -> Self {
        self.struct_variant_name = Some(name);
        self
    }

    #[inline]
    fn add_key(&mut self, term: Term<'a>) -> Result<(), Error> {
        if self.keys.len() == self.values.len() {
            self.keys.push(term);
            Ok(())
        } else {
            Err(Error::SerializationError(
                "MapSerializer::add_key called twice in a row".into(),
            ))
        }
    }

    #[inline]
    fn add_val(&mut self, term: Term<'a>) -> Result<(), Error> {
        if self.keys.len() == self.values.len() + 1 {
            self.values.push(term);
            Ok(())
        } else {
            Err(Error::SerializationError(
                "MapSerializer::add_val called incorrectly".into(),
            ))
        }
    }

    #[inline]
    fn to_map(&self) -> Result<Term<'a>, Error> {
        if let Some(name) = self.struct_variant_name {
            let __struct__ = atoms::__struct__().to_term(self.ser.env);
            let mut keys = Vec::with_capacity(self.keys.len() + 1);
            let mut values = Vec::with_capacity(self.values.len() + 1);
            keys.push(__struct__);
            values.push(name);
            keys.extend_from_slice(&self.keys);
            values.extend_from_slice(&self.values);
            Term::map_from_arrays(self.ser.env, &keys, &values).map_err(|_| Error::InvalidMap)
        } else {
            Term::map_from_arrays(self.ser.env, &self.keys, &self.values)
                .map_err(|_| Error::InvalidMap)
        }
    }
}

impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = Term<'a>;
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.add_key(key.serialize(self.ser)?)?;
        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        self.add_val(value.serialize(self.ser)?)?;
        Ok(())
    }

    fn end(self) -> Result<Term<'a>, Error> {
        self.to_map()
    }
}

impl<'a> ser::SerializeStruct for MapSerializer<'a> {
    type Ok = Term<'a>;
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        let key_term = key.encode(self.ser.env);
        self.add_key(key_term)?;
        self.add_val(value.serialize(self.ser)?)?;
        Ok(())
    }

    fn end(self) -> Result<Term<'a>, Error> {
        self.to_map()
    }
}

impl<'a> ser::SerializeStructVariant for MapSerializer<'a> {
    type Ok = Term<'a>;
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Term<'a>, Error> {
        ser::SerializeStruct::end(self)
    }
}
