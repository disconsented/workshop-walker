mod size_hint;

use std::{borrow::Cow, fmt, marker::PhantomData};

use serde::{
    Deserialize, Deserializer,
    de::{
        DeserializeSeed, EnumAccess, Error, Expected, MapAccess, SeqAccess, Unexpected,
        VariantAccess, Visitor,
    },
};
use serde_content::{Number, Value};

// I'm sorry for anyone that reads this, this is a clone of serdes private
// ContentRefDeserializer, it's needed to implement the `Source<T>` enum

pub struct ValueRefDeserializer<'a, 'de: 'a, E> {
    value: &'a Value<'de>,
    err: PhantomData<E>,
}

impl<'a, 'de, E> ValueRefDeserializer<'a, 'de, E>
where
    E: Error,
{
    #[cold]
    fn invalid_type(self, exp: &dyn Expected) -> E {
        E::invalid_type(value_unexpected(self.value), exp)
    }

    fn deserialize_integer<V>(self, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::Number(Number::U8(v)) => visitor.visit_u8(v),
            Value::Number(Number::U16(v)) => visitor.visit_u16(v),
            Value::Number(Number::U32(v)) => visitor.visit_u32(v),
            Value::Number(Number::U64(v)) => visitor.visit_u64(v),
            Value::Number(Number::I8(v)) => visitor.visit_i8(v),
            Value::Number(Number::I16(v)) => visitor.visit_i16(v),
            Value::Number(Number::I32(v)) => visitor.visit_i32(v),
            Value::Number(Number::I64(v)) => visitor.visit_i64(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_float<V>(self, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::Number(Number::F32(v)) => visitor.visit_f32(v),
            Value::Number(Number::F64(v)) => visitor.visit_f64(v),
            Value::Number(Number::U8(v)) => visitor.visit_u8(v),
            Value::Number(Number::U16(v)) => visitor.visit_u16(v),
            Value::Number(Number::U32(v)) => visitor.visit_u32(v),
            Value::Number(Number::U64(v)) => visitor.visit_u64(v),
            Value::Number(Number::I8(v)) => visitor.visit_i8(v),
            Value::Number(Number::I16(v)) => visitor.visit_i16(v),
            Value::Number(Number::I32(v)) => visitor.visit_i32(v),
            Value::Number(Number::I64(v)) => visitor.visit_i64(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }
}

fn visit_value_seq_ref<'a, 'de, V, E>(_: &'a [Value<'de>], _: V) -> Result<V::Value, E>
where
    V: Visitor<'de>,
    E: Error,
{
    unimplemented!()
    // let mut seq_visitor = SeqRefDeserializer::new(value);
    // let value = ?visitor.visit_seq(&mut seq_visitor)?;
    // ?seq_visitor.end()?;
    // Ok(value)
}

fn visit_value_map_ref<'a, 'de, V, E>(
    value: &'a [(Value<'de>, Value<'de>)],
    visitor: V,
) -> Result<V::Value, E>
where
    V: Visitor<'de>,
    E: Error,
{
    let mut map_visitor = MapRefDeserializer::new(value);
    let value = visitor.visit_map(&mut map_visitor)?;
    map_visitor.end()?;
    Ok(value)
}

/// Used when deserializing an untagged enum because the value may need
/// to be used more than once.
impl<'de, 'a, E> Deserializer<'de> for ValueRefDeserializer<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Bool(v) => visitor.visit_bool(*v),
            Value::Number(Number::U8(v)) => visitor.visit_u8(*v),
            Value::Number(Number::U16(v)) => visitor.visit_u16(*v),
            Value::Number(Number::U32(v)) => visitor.visit_u32(*v),
            Value::Number(Number::U64(v)) => visitor.visit_u64(*v),
            Value::Number(Number::I8(v)) => visitor.visit_i8(*v),
            Value::Number(Number::I16(v)) => visitor.visit_i16(*v),
            Value::Number(Number::I32(v)) => visitor.visit_i32(*v),
            Value::Number(Number::I64(v)) => visitor.visit_i64(*v),
            Value::Number(Number::F32(v)) => visitor.visit_f32(*v),
            Value::Number(Number::F64(v)) => visitor.visit_f64(*v),
            Value::Char(v) => visitor.visit_char(*v),
            Value::String(Cow::Owned(v)) => visitor.visit_string(v.clone()),
            Value::String(Cow::Borrowed(v)) => visitor.visit_borrowed_str(v),
            // Value::ByteBuf(ref v) => visitor.visit_bytes(v),
            // Value::Bytes(v) => visitor.visit_borrowed_bytes(v),
            Value::Unit => visitor.visit_unit(),
            // Value::None => visitor.visit_none(),
            // Value::Some(ref v) => visitor.visit_some(ValueRefDeserializer::new(v)),
            // Value::Newtype(ref v) => visitor.visit_newtype_struct(ValueRefDeserializer::new(v)),
            Value::Seq(_) => {
                unimplemented!();
                // visit_value_seq_ref(v, visitor)
            }
            Value::Map(v) => visit_value_map_ref(v, visitor),
            _ => unimplemented!(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_integer(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_float(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_float(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::Char(v) => visitor.visit_char(v),
            Value::String(ref v) => visitor.visit_str(v),
            // Value::Str(v) => visitor.visit_borrowed_str(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::String(ref v) => visitor.visit_str(v),
            // Value::Str(v) => visitor.visit_borrowed_str(v),
            // Value::ByteBuf(ref v) => visitor.visit_bytes(v),
            // Value::Bytes(v) => visitor.visit_borrowed_bytes(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::String(ref v) => visitor.visit_str(v),
            // Value::Str(v) => visitor.visit_borrowed_str(v),
            // Value::ByteBuf(ref v) => visitor.visit_bytes(v),
            // Value::Bytes(v) => visitor.visit_borrowed_bytes(v),
            Value::Seq(ref v) => visit_value_seq_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        // Covered by tests/test_enum_untagged.rs
        //      with_optional_field::*
        match *self.value {
            // Value::None => visitor.visit_none(),
            // Value::Some(ref v) => visitor.visit_some(ValueRefDeserializer::new(v)),
            Value::Unit => visitor.visit_unit(),
            // This case is to support data formats which do not encode an
            // indication whether a value is optional. An example of such a
            // format is JSON, and a counterexample is RON. When requesting
            // `deserialize_any` in JSON, the data format never performs
            // `Visitor::visit_some` but we still must be able to
            // deserialize the resulting Value into data structures with
            // optional fields.
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::Unit => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

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

    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value, E>
    where
        V: Visitor<'de>,
    {
        // Covered by tests/test_enum_untagged.rs
        //      newtype_struct
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::Seq(ref v) => visit_value_seq_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::Map(ref v) => visit_value_map_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::Seq(ref v) => visit_value_seq_ref(v, visitor),
            Value::Map(ref v) => visit_value_map_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match *self.value {
            Value::Map(ref value) => {
                let mut iter = value.iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(Error::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant, Some(value))
            }
            ref s @ Value::String(_) => (s, None),
            ref other => {
                return Err(Error::invalid_type(
                    value_unexpected(other),
                    &"string or map",
                ));
            }
        };
        visitor.visit_enum(EnumRefDeserializer {
            variant,
            value,
            err: PhantomData,
        })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self.value {
            Value::String(ref v) => visitor.visit_str(v),
            // Value::Str(v) => visitor.visit_borrowed_str(v),
            // Value::ByteBuf(ref v) => visitor.visit_bytes(v),
            // Value::Bytes(v) => visitor.visit_borrowed_bytes(v),
            Value::Number(Number::U8(v)) => visitor.visit_u8(v),
            Value::Number(Number::U64(v)) => visitor.visit_u64(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    // fn __deserialize_value_v1<V>(self, visitor: V) -> Result<V::Value,
    // Self::Error> where
    //     V: Visitor<'de, Value = Value<'de>>,
    // {
    //     let _ = visitor;
    //     Ok(value_clone(self.value))
    // }
}

impl<'a, 'de, E> ValueRefDeserializer<'a, 'de, E> {
    /// private API, don't use
    pub fn new(value: &'a Value<'de>) -> Self {
        ValueRefDeserializer {
            value,
            err: PhantomData,
        }
    }
}

impl<'a, 'de: 'a, E> Copy for ValueRefDeserializer<'a, 'de, E> {}

impl<'a, 'de: 'a, E> Clone for ValueRefDeserializer<'a, 'de, E> {
    fn clone(&self) -> Self {
        *self
    }
}

fn value_unexpected<'a>(value: &'a Value<'_>) -> Unexpected<'a> {
    match *value {
        Value::Bool(b) => Unexpected::Bool(b),
        Value::Number(Number::U8(n)) => Unexpected::Unsigned(n as u64),
        Value::Number(Number::U16(n)) => Unexpected::Unsigned(n as u64),
        Value::Number(Number::U32(n)) => Unexpected::Unsigned(n as u64),
        Value::Number(Number::U64(n)) => Unexpected::Unsigned(n),
        Value::Number(Number::I8(n)) => Unexpected::Signed(n as i64),
        Value::Number(Number::I16(n)) => Unexpected::Signed(n as i64),
        Value::Number(Number::I32(n)) => Unexpected::Signed(n as i64),
        Value::Number(Number::I64(n)) => Unexpected::Signed(n),
        Value::Number(Number::F32(f)) => Unexpected::Float(f as f64),
        Value::Number(Number::F64(f)) => Unexpected::Float(f),
        Value::Char(c) => Unexpected::Char(c),
        Value::String(ref s) => Unexpected::Str(s),
        // Value::String(s) => Unexpected::Str(s),
        // Value::ByteBuf(ref b) => Unexpected::Bytes(b),
        // Value::Bytes(b) => Unexpected::Bytes(b),
        // Value::None | Value::Some(_) => Unexpected::Option,
        Value::Unit => Unexpected::Unit,
        // Value::Newtype(_) => Unexpected::NewtypeStruct,
        Value::Seq(_) => Unexpected::Seq,
        Value::Map(_) => Unexpected::Map,
        _ => unimplemented!(),
    }
}

struct MapRefDeserializer<'a, 'de, E> {
    iter: <&'a [(Value<'de>, Value<'de>)] as IntoIterator>::IntoIter,
    value: Option<&'a Value<'de>>,
    count: usize,
    error: PhantomData<E>,
}

impl<'a, 'de, E> MapRefDeserializer<'a, 'de, E> {
    fn new(value: &'a [(Value<'de>, Value<'de>)]) -> Self {
        MapRefDeserializer {
            iter: value.iter(),
            value: None,
            count: 0,
            error: PhantomData,
        }
    }
}

impl<'a, 'de, E> MapRefDeserializer<'a, 'de, E>
where
    E: Error,
{
    fn end(self) -> Result<(), E> {
        let remaining = self.iter.count();
        if remaining == 0 {
            Ok(())
        } else {
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(Error::invalid_length(
                self.count + remaining,
                &ExpectedInMap(self.count),
            ))
        }
    }
}

impl<'a, 'de, E> MapRefDeserializer<'a, 'de, E> {
    fn next_pair(&mut self) -> Option<(&'a Value<'de>, &'a Value<'de>)> {
        match self.iter.next() {
            Some((k, v)) => {
                self.count += 1;
                Some((k, v))
            }
            None => None,
        }
    }
}

impl<'a, 'de, E> Deserializer<'de> for MapRefDeserializer<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple_struct map
        struct enum identifier ignored_any
    }

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_map(&mut self)?;
        self.end()?;
        Ok(value)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = visitor.visit_seq(&mut self)?;
        self.end()?;
        Ok(value)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let _ = len;
        self.deserialize_seq(visitor)
    }
}

impl<'a, 'de, E> MapAccess<'de> for MapRefDeserializer<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.next_pair() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(ValueRefDeserializer::new(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let value = self.value.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let value = value.expect("MapAccess::next_value called before next_key");
        seed.deserialize(ValueRefDeserializer::new(value))
    }

    fn next_entry_seed<TK, TV>(
        &mut self,
        kseed: TK,
        vseed: TV,
    ) -> Result<Option<(TK::Value, TV::Value)>, Self::Error>
    where
        TK: DeserializeSeed<'de>,
        TV: DeserializeSeed<'de>,
    {
        match self.next_pair() {
            Some((key, value)) => {
                let key = kseed.deserialize(ValueRefDeserializer::new(key))?;
                let value = vseed.deserialize(ValueRefDeserializer::new(value))?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

impl<'a, 'de, E> SeqAccess<'de> for MapRefDeserializer<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.next_pair() {
            Some((k, v)) => {
                let de = PairRefDeserializer(k, v, PhantomData);
                seed.deserialize(de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

#[allow(dead_code, reason = "false negative")]
struct PairRefDeserializer<'a, 'de, E>(&'a Value<'de>, &'a Value<'de>, PhantomData<E>);

impl<'a, 'de, E> Deserializer<'de> for PairRefDeserializer<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct tuple_struct map
        struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
        // let mut pair_visitor = PairRefVisitor(Some(self.0), Some(self.1),
        // PhantomData); let pair = visitor.visit_seq(&mut
        // pair_visitor)?; if pair_visitor.1.is_none() {
        //     Ok(pair)
        // } else {
        //     let remaining = pair_visitor.size_hint().unwrap();
        //     // First argument is the number of elements in the data, second
        //     // argument is the number of elements expected by the
        // Deserialize.     Err(Error::invalid_length(2,
        // &ExpectedInSeq(2 - remaining))) }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if len == 2 {
            self.deserialize_seq(visitor)
        } else {
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(Error::invalid_length(2, &ExpectedInSeq(len)))
        }
    }
}

#[allow(dead_code, reason = "Used in commented out code; kept for the future")]
struct PairRefVisitor<'a, 'de, E>(
    Option<&'a Value<'de>>,
    Option<&'a Value<'de>>,
    PhantomData<E>,
);

struct ExpectedInMap(usize);

impl Expected for ExpectedInMap {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 1 {
            formatter.write_str("1 element in map")
        } else {
            write!(formatter, "{} elements in map", self.0)
        }
    }
}

struct ExpectedInSeq(usize);

impl Expected for ExpectedInSeq {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 1 {
            formatter.write_str("1 element in sequence")
        } else {
            write!(formatter, "{} elements in sequence", self.0)
        }
    }
}

struct EnumRefDeserializer<'a, 'de: 'a, E>
where
    E: Error,
{
    variant: &'a Value<'de>,
    value: Option<&'a Value<'de>>,
    err: PhantomData<E>,
}

impl<'de, 'a, E> EnumAccess<'de> for EnumRefDeserializer<'a, 'de, E>
where
    E: Error,
{
    type Error = E;
    type Variant = VariantRefDeserializer<'a, 'de, Self::Error>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let visitor = VariantRefDeserializer {
            value: self.value,
            err: PhantomData,
        };
        seed.deserialize(ValueRefDeserializer::new(self.variant))
            .map(|v| (v, visitor))
    }
}

struct VariantRefDeserializer<'a, 'de: 'a, E>
where
    E: Error,
{
    value: Option<&'a Value<'de>>,
    err: PhantomData<E>,
}

impl<'de, 'a, E> VariantAccess<'de> for VariantRefDeserializer<'a, 'de, E>
where
    E: Error,
{
    type Error = E;

    fn unit_variant(self) -> Result<(), E> {
        match self.value {
            Some(value) => Deserialize::deserialize(ValueRefDeserializer::new(value)),
            // Covered by tests/test_annotations.rs
            //      test_partially_untagged_adjacently_tagged_enum
            // Covered by tests/test_enum_untagged.rs
            //      newtype_enum::unit
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, E>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            // Covered by tests/test_annotations.rs
            //      test_partially_untagged_enum_desugared
            //      test_partially_untagged_enum_generic
            // Covered by tests/test_enum_untagged.rs
            //      newtype_enum::newtype
            Some(value) => seed.deserialize(ValueRefDeserializer::new(value)),
            None => Err(Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            // Covered by tests/test_annotations.rs
            //      test_partially_untagged_enum
            //      test_partially_untagged_enum_desugared
            // Covered by tests/test_enum_untagged.rs
            //      newtype_enum::tuple0
            //      newtype_enum::tuple2
            Some(Value::Seq(v)) => visit_value_seq_ref(v, visitor),
            Some(other) => Err(Error::invalid_type(
                value_unexpected(other),
                &"tuple variant",
            )),
            None => Err(Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            // Covered by tests/test_enum_untagged.rs
            //      newtype_enum::struct_from_map
            Some(Value::Map(v)) => visit_value_map_ref(v, visitor),
            // Covered by tests/test_enum_untagged.rs
            //      newtype_enum::struct_from_seq
            //      newtype_enum::empty_struct_from_seq
            Some(Value::Seq(v)) => visit_value_seq_ref(v, visitor),
            Some(other) => Err(Error::invalid_type(
                value_unexpected(other),
                &"struct variant",
            )),
            None => Err(Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}
