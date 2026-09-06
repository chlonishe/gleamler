use crate::serde::{atoms, error::Error, util};
use crate::{
    Encoder, Term, TermType,
    types::{ListIterator, MapIterator},
};
use serde::{
    de::{
        self, Deserialize, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess,
        Visitor,
    },
    forward_to_deserialize_any,
};
use std::iter;

#[inline]
pub fn from_term<'de, 'a: 'de, T>(term: Term<'a>) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    T::deserialize(Deserializer::from(term))
}

pub struct Deserializer<'a> {
    term: Term<'a>,
}

impl<'a> From<Term<'a>> for Deserializer<'a> {
    fn from(term: Term<'a>) -> Deserializer<'a> {
        Deserializer { term }
    }
}

macro_rules! try_parse_number {
    ($term:expr, $type:ty, $visitor:expr, $visit_fn:ident) => {
        if let Ok(num) = util::parse_number::<$type>(&$term) {
            return $visitor.$visit_fn(num);
        }
    };
}

impl<'de, 'a: 'de> de::Deserializer<'de> for Deserializer<'a> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.term.get_type() {
            TermType::Atom => {
                if util::is_nil(&self.term) {
                    self.deserialize_unit(visitor)
                } else if let Ok(b) = util::parse_bool(&self.term) {
                    visitor.visit_bool(b)
                } else {
                    let string = atoms::term_to_string(&self.term)?;
                    visitor.visit_string(string)
                }
            }
            TermType::Integer => {
                try_parse_number!(self.term, u64, visitor, visit_u64);
                try_parse_number!(self.term, i64, visitor, visit_i64);
                try_parse_number!(self.term, u128, visitor, visit_u128);
                try_parse_number!(self.term, i128, visitor, visit_i128);
                Err(Error::ExpectedNumber)
            }
            TermType::Float => {
                try_parse_number!(self.term, f64, visitor, visit_f64);
                Err(Error::ExpectedNumber)
            }
            TermType::Binary => {
                let bytes = util::parse_binary(self.term)?;
                match std::str::from_utf8(bytes) {
                    Ok(s) => visitor.visit_borrowed_str(s),
                    Err(_) => visitor.visit_borrowed_bytes(bytes),
                }
            }
            TermType::List => self.deserialize_seq(visitor),
            TermType::Map => {
                let iter = MapIterator::new(self.term).ok_or(Error::ExpectedMap)?;
                visitor.visit_map(MapDeserializer::new(iter))
            }
            TermType::Tuple => {
                let tuple = util::validate_tuple(self.term, None)?;
                visitor.visit_seq(SequenceDeserializer::new(tuple.into_iter()))
            }
            _ => Err(Error::TypeHintsRequired),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.term.is_empty_list() || util::is_nil(&self.term) {
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNil)
        }
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(util::parse_bool(&self.term)?)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if util::is_none(&self.term) {
            visitor.visit_none()
        } else if let Ok((tag, inner)) = self.term.decode::<(crate::types::Atom, Term<'a>)>() {
            if tag == atoms::some() {
                visitor.visit_some(Deserializer::from(inner))
            } else {
                visitor.visit_some(self)
            }
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i128(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u128(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(util::parse_number(&self.term)?)
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let s: &str = util::parse_str(self.term)?;
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(Error::ExpectedChar),
        }
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(util::parse_str(self.term)?)
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(util::parse_binary(self.term)?)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(util::parse_binary(self.term)?)
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let tuple = util::validate_tuple(self.term, Some(2))?;
        let name_term = atoms::str_to_term(self.term.get_env(), name)?;
        if tuple[0].ne(&name_term) {
            return Err(Error::InvalidStructName);
        }
        visitor.visit_newtype_struct(Deserializer::from(tuple[1]))
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.term.is_list() || self.term.is_empty_list() {
            let iter: ListIterator = self.term.decode().map_err(|_| Error::ExpectedList)?;
            let vec: Vec<Term<'a>> = iter
                .map(|r| r.map_err(|_| Error::InvalidList))
                .collect::<Result<Vec<_>, _>>()?;
            visitor.visit_seq(SequenceDeserializer::new(vec.into_iter()))
        } else if self.term.is_binary() {
            let binary = self
                .term
                .decode_as_binary()
                .map_err(|_| Error::ExpectedBinary)?;
            let iter = binary
                .as_slice()
                .iter()
                .map(|x| x.encode(self.term.get_env()));
            visitor.visit_seq(SequenceDeserializer::new(iter))
        } else {
            Err(Error::ExpectedList)
        }
    }

    #[inline]
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let tuple = util::validate_tuple(self.term, Some(len))?;
        visitor.visit_seq(SequenceDeserializer::new(tuple.into_iter()))
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut tuple = util::validate_tuple(self.term, Some(len + 1))?;
        let name_term = atoms::str_to_term(self.term.get_env(), name)?;
        if tuple[0].ne(&name_term) {
            return Err(Error::InvalidStructName);
        }
        let iter = tuple.split_off(1).into_iter();
        visitor.visit_seq(SequenceDeserializer::new(iter))
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.term.is_map() {
            let iter = MapIterator::new(self.term).ok_or(Error::ExpectedMap)?;
            visitor.visit_map(MapDeserializer::new(iter))
        } else {
            Err(Error::ExpectedMap)
        }
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.term.is_map() {
            let iter = MapIterator::new(self.term).ok_or(Error::ExpectedStruct)?;
            visitor.visit_map(MapDeserializer::new(iter))
        } else {
            Err(Error::ExpectedStruct)
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        use EnumDeserializerType as EnumType;

        let is_result = _name == "Result" || _name.ends_with("::Result");

        let variant: Option<(EnumType, Term<'a>)> = match self.term.get_type() {
            TermType::Atom => Some((EnumType::Unit, self.term)),
            TermType::Binary => Some((EnumType::Unit, self.term)),
            TermType::Tuple => {
                let tuple = util::validate_tuple(self.term, None)?;
                match tuple.len() {
                    0 | 1 => None,
                    2 => Some((EnumType::Newtype, tuple[0])),
                    _ => Some((EnumType::Tuple, tuple[0])),
                }
            }
            TermType::Map => {
                let __struct__ = atoms::__struct__().to_term(self.term.get_env());
                if let Ok(struct_name_term) = self.term.map_get(__struct__) {
                    Some((EnumType::Struct, struct_name_term))
                } else {
                    None
                }
            }
            _ => None,
        };

        variant.ok_or(Error::ExpectedEnum).and_then(|variant| {
            let (vtype, term) = variant;
            let enum_de = EnumDeserializer::new(vtype, term, variants, Some(self.term), is_result)?;
            visitor.visit_enum(enum_de)
        })
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.term.get_type() {
            TermType::Atom => self.deserialize_str(visitor),
            TermType::Binary => self.deserialize_str(visitor),
            TermType::Integer => self.deserialize_i64(visitor),
            _ => Err(Error::ExpectedAtom),
        }
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

/// SequenceDeserializer
pub struct SequenceDeserializer<'a, I>
where
    I: Iterator<Item = Term<'a>>,
{
    iter: iter::Fuse<I>,
}

impl<'a, I> SequenceDeserializer<'a, I>
where
    I: Iterator<Item = Term<'a>>,
{
    #[inline]
    fn new(iter: I) -> Self {
        SequenceDeserializer { iter: iter.fuse() }
    }
}

impl<'de, 'a: 'de, I> SeqAccess<'de> for SequenceDeserializer<'a, I>
where
    I: Iterator<Item = Term<'a>>,
{
    type Error = Error;

    #[inline]
    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>, Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            None => Ok(None),
            Some(term) => seed.deserialize(Deserializer::from(term)).map(Some),
        }
    }
}

/// MapDeserializer
pub struct MapDeserializer<'a, I>
where
    I: Iterator,
{
    iter: iter::Fuse<I>,
    current_value: Option<Term<'a>>,
}

impl<'a, I> MapDeserializer<'a, I>
where
    I: Iterator,
{
    #[inline]
    fn new(iter: I) -> Self {
        MapDeserializer {
            iter: iter.fuse(),
            current_value: None,
        }
    }
}

impl<'de, 'a: 'de, I> MapAccess<'de> for MapDeserializer<'a, I>
where
    I: Iterator<Item = (Term<'a>, Term<'a>)>,
{
    type Error = Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.current_value.is_some() {
            return Err(Error::DeserializationError(
                "MapDeserializer::next_key_seed called twice in a row".into(),
            ));
        }

        self.iter.next().map_or(Ok(None), |pair| {
            let (key, value) = pair;
            self.current_value = Some(value);
            seed.deserialize(Deserializer::from(key)).map(Some)
        })
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.current_value {
            None => Err(Error::ExpectedStructValue),
            Some(value) => {
                self.current_value = None;
                seed.deserialize(Deserializer::from(value))
            }
        }
    }
}

/// EnumDeserializerType
pub enum EnumDeserializerType {
    #[allow(dead_code)]
    Any,
    Unit,
    Newtype,
    Tuple,
    Struct,
}

/// EnumDeserializer
pub struct EnumDeserializer<'a> {
    variant_type: EnumDeserializerType,
    variant_term: Term<'a>,
    term: Option<Term<'a>>,
    is_result: bool,
}

impl<'a> EnumDeserializer<'a> {
    #[inline]
    fn new(
        variant_type: EnumDeserializerType,
        variant_term: Term<'a>,
        variants: &'static [&'static str],
        term: Option<Term<'a>>,
        is_result: bool,
    ) -> Result<Self, Error> {
        let var_de = VariantNameDeserializer::from(variant_term, is_result);
        let variant = String::deserialize(var_de).or(Err(Error::InvalidVariantName))?;

        match variant_type {
            EnumDeserializerType::Any => Ok(EnumDeserializer {
                variant_type,
                variant_term,
                term,
                is_result,
            }),
            _ => {
                if variants.contains(&variant.as_str()) {
                    Ok(EnumDeserializer {
                        variant_type,
                        variant_term,
                        term,
                        is_result,
                    })
                } else {
                    Err(Error::InvalidVariantName)
                }
            }
        }
    }
}

impl<'de, 'a: 'de> EnumAccess<'de> for EnumDeserializer<'a> {
    type Error = Error;
    type Variant = Self;

    #[inline]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let var_de = VariantNameDeserializer::from(self.variant_term, self.is_result);
        let val = seed.deserialize(var_de)?;
        Ok((val, self))
    }
}

impl<'de, 'a: 'de> VariantAccess<'de> for EnumDeserializer<'a> {
    type Error = Error;

    #[inline]
    fn unit_variant(self) -> Result<(), Error> {
        match self.variant_type {
            EnumDeserializerType::Any | EnumDeserializerType::Unit => Ok(()),
            _ => Err(Error::ExpectedUnitVariant),
        }
    }

    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.variant_type {
            EnumDeserializerType::Any | EnumDeserializerType::Newtype => {
                if let Some(term) = self.term {
                    let tuple = util::validate_tuple(term, Some(2))?;
                    seed.deserialize(Deserializer::from(tuple[1]))
                } else {
                    Err(Error::ExpectedNewtypeVariant)
                }
            }
            _ => Err(Error::ExpectedNewtypeVariant),
        }
    }

    #[inline]
    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.variant_type {
            EnumDeserializerType::Any | EnumDeserializerType::Tuple => {
                if let Some(term) = self.term {
                    let mut tuple = util::validate_tuple(term, Some(len + 1))?;
                    let iter = tuple.split_off(1).into_iter();
                    visitor.visit_seq(SequenceDeserializer::new(iter))
                } else {
                    Err(Error::ExpectedTupleVariant)
                }
            }
            _ => Err(Error::ExpectedTupleVariant),
        }
    }

    #[inline]
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.variant_type {
            EnumDeserializerType::Struct => {
                if let Some(term) = self.term {
                    if !term.is_map() {
                        return Err(Error::ExpectedStruct);
                    }
                    let __struct__ = atoms::__struct__().to_term(term.get_env());
                    let iter = MapIterator::new(term)
                        .ok_or(Error::ExpectedStruct)?
                        .filter(|(k, _)| *k != __struct__);
                    visitor.visit_map(MapDeserializer::new(iter))
                } else {
                    Err(Error::ExpectedStructVariant)
                }
            }
            _ => Err(Error::ExpectedStructVariant),
        }
    }
}

/// Deserializer for atoms and map keys
pub struct VariantNameDeserializer<'a> {
    variant: Term<'a>,
    is_result: bool,
}

impl<'a> VariantNameDeserializer<'a> {
    pub fn from(variant: Term<'a>, is_result: bool) -> Self {
        VariantNameDeserializer { variant, is_result }
    }
}

impl<'a> From<Term<'a>> for VariantNameDeserializer<'a> {
    fn from(variant: Term<'a>) -> Self {
        Self::from(variant, false)
    }
}

impl<'de, 'a: 'de> de::Deserializer<'de> for VariantNameDeserializer<'a> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.variant.get_type() {
            TermType::Atom => {
                let string =
                    atoms::term_to_string(&self.variant).or(Err(Error::InvalidVariantName))?;
                let string = if self.is_result {
                    match string.as_str() {
                        "ok" => "Ok".to_string(),
                        "error" => "Err".to_string(),
                        _ => string,
                    }
                } else {
                    string
                };
                visitor.visit_string(string)
            }
            TermType::Binary => visitor.visit_string(util::term_to_str(&self.variant)?),
            TermType::Integer => visitor.visit_string(util::term_to_str(&self.variant)?),
            TermType::Float => visitor.visit_string(util::term_to_str(&self.variant)?),
            _ => Err(Error::ExpectedStringable),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
    }
}
