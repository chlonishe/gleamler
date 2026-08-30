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
    use std::collections::{BTreeSet, HashSet, LinkedList, VecDeque};

    #[test]
    fn hashset_roundtrip_via_vec() {
        let original: HashSet<i64> = [1, 2, 3].into_iter().collect();
        let vec: Vec<i64> = original.iter().copied().collect();
        let reconstructed: HashSet<i64> = vec.into_iter().collect();
        assert_eq!(original, reconstructed);
    }

    #[test]
    fn btreeset_maintains_order() {
        let set: BTreeSet<i32> = [3, 1, 2].into_iter().collect();
        let vec: Vec<i32> = set.iter().copied().collect();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn vecdeque_front_back() {
        let mut dq = VecDeque::new();
        dq.push_back(1);
        dq.push_front(0);
        assert_eq!(dq.as_slices().0, &[0]);
        assert_eq!(dq.as_slices().1, &[1]);
    }

    #[test]
    fn linkedlist_collect_identity() {
        let original = LinkedList::from([1, 2, 3]);
        let vec: Vec<i32> = original.into_iter().collect();
        let reconstructed: LinkedList<i32> = vec.into_iter().collect();
        assert_eq!(reconstructed, LinkedList::from([1, 2, 3]));
    }

    #[test]
    fn hashset_deduplicates_on_reconstruct() {
        let vec = vec![1, 1, 2, 2, 3];
        let set: HashSet<i64> = vec.into_iter().collect();
        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
    }
}
