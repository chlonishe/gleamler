use std::collections::{BTreeSet, HashSet, LinkedList, VecDeque};

use crate::{Decoder, Encoder, Env, NifResult, Term};

// HashSet

impl<T> Encoder for HashSet<T>
where
    T: Encoder + Eq + std::hash::Hash,
{
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let vec: Vec<&T> = self.iter().collect();
        vec.encode(env)
    }
}

impl<'a, T> Decoder<'a> for HashSet<T>
where
    T: Decoder<'a> + Eq + std::hash::Hash,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let vec: Vec<T> = term.decode()?;
        Ok(vec.into_iter().collect())
    }
}

// BTreeSet

impl<T> Encoder for BTreeSet<T>
where
    T: Encoder + Ord,
{
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let vec: Vec<&T> = self.iter().collect();
        vec.encode(env)
    }
}

impl<'a, T> Decoder<'a> for BTreeSet<T>
where
    T: Decoder<'a> + Ord,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let vec: Vec<T> = term.decode()?;
        Ok(vec.into_iter().collect())
    }
}

// VecDeque

impl<T> Encoder for VecDeque<T>
where
    T: Encoder,
{
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let vec: Vec<&T> = self.iter().collect();
        vec.encode(env)
    }
}

impl<'a, T> Decoder<'a> for VecDeque<T>
where
    T: Decoder<'a>,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let vec: Vec<T> = term.decode()?;
        Ok(vec.into())
    }
}

// LinkedList

impl<T> Encoder for LinkedList<T>
where
    T: Encoder,
{
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let vec: Vec<&T> = self.iter().collect();
        vec.encode(env)
    }
}

impl<'a, T> Decoder<'a> for LinkedList<T>
where
    T: Decoder<'a>,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let vec: Vec<T> = term.decode()?;
        Ok(vec.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet, LinkedList, VecDeque};

    #[test]
    fn trait_impls_exist() {
        fn assert_enc<T: Encoder>() {}
        fn assert_dec<'a, T: Decoder<'a>>() {}

        assert_enc::<HashSet<i64>>();
        assert_dec::<HashSet<i64>>();
        assert_enc::<BTreeSet<i64>>();
        assert_dec::<BTreeSet<i64>>();
        assert_enc::<VecDeque<String>>();
        assert_dec::<VecDeque<String>>();
        assert_enc::<LinkedList<bool>>();
        assert_dec::<LinkedList<bool>>();
    }
}
