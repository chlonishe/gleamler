//! Utilities used to access and create Erlang maps.

use crate::wrapper::{ map, NIF_TERM };
use crate::{Decoder, Encoder, Env, Error, NifResult, Term};

#[inline]
pub fn map_new(env: Env) -> Term {
    unsafe { Term::new(env, map::map_new(env.as_c_arg())) }
}

/// ## Map terms
impl<'a> Term<'a> {
    /// Constructs a new, empty map term.
    ///
    /// ### Erlang equivalent
    /// ```erlang
    /// #{}
    /// ```
    #[inline]
    pub fn map_new(env: Env<'a>) -> Term<'a> {
        map_new(env)
    }

    /// Construct a new map from two vectors
    ///
    /// ### Erlang equivalent
    /// ```erlang
    /// maps:from_list(lists:zip(Keys, Values))
    /// ```
    #[inline]
    pub fn map_from_arrays(
        env: Env<'a>,
        keys: &[impl Encoder],
        values: &[impl Encoder],
    ) -> NifResult<Term<'a>> {
        if keys.len() != values.len() { return Err(Error::BadArg); }
        let mut k = Vec::with_capacity(keys.len());
        let mut v = Vec::with_capacity(values.len());
        for i in 0..keys.len() {
            k.push(keys[i].encode(env).as_c_arg());
            v.push(values[i].encode(env).as_c_arg());
        }
        unsafe {
            map::make_map_from_arrays(env.as_c_arg(), &k, &v)
                .map_or_else(|| Err(Error::BadArg), |map| Ok(Term::new(env, map)))
        }
    }

    pub fn map_from_raw_arrays(
        env: Env<'a>,
        keys: &[NIF_TERM],
        values: &[NIF_TERM],
    ) -> NifResult<Term<'a>> {
        if keys.len() == values.len() {
            unsafe {
                map::make_map_from_arrays(env.as_c_arg(), keys, values)
                    .map_or_else(|| Err(Error::BadArg), |m| Ok(Term::new(env, m)))
            }
        } else {
            Err(Error::BadArg)
        }
    }

    /// Construct a new map from two vectors of terms.
    ///
    /// It is identical to map_from_arrays, but requires the keys and values to
    /// be encoded already - this is useful for constructing maps whose values
    /// or keys are different Rust types, with the same performance as map_from_arrays.
    pub fn map_from_term_arrays(
        env: Env<'a>,
        keys: &[Term<'a>],
        values: &[Term<'a>],
    ) -> NifResult<Term<'a>> {
        if keys.len() == values.len() {
            let keys: Vec<_> = keys.iter().map(|k| k.as_c_arg()).collect();
            let values: Vec<_> = values.iter().map(|v| v.as_c_arg()).collect();
            Self::map_from_raw_arrays(env, &keys, &values)
        } else {
            Err(Error::BadArg)
        }
    }

    /// Construct a new map from pairs of terms
    ///
    /// It is similar to `map_from_arrays` but
    /// receives only one vector with the pairs
    /// of `(key, value)`.
    ///
    /// ### Erlang equivalent
    /// ```erlang
    /// maps:from_list([{<<"foo">>, 1}, {<<"bar">>, 2}])
    /// ```
    #[inline]
    pub fn map_from_pairs(
        env: Env<'a>,
        pairs: &[(impl Encoder, impl Encoder)],
    ) -> NifResult<Term<'a>> {
        let mut keys = Vec::with_capacity(pairs.len());
        let mut values = Vec::with_capacity(pairs.len());
        for (k, v) in pairs {
            keys.push(k.encode(env).as_c_arg());
            values.push(v.encode(env).as_c_arg());
        }
        unsafe {
            map::make_map_from_arrays(env.as_c_arg(), &keys, &values)
                .map_or_else(|| Err(Error::BadArg), |map| Ok(Term::new(env, map)))
        }
    }

    /// Gets the value corresponding to a key in a map term.
    ///
    /// Returns Err(Error::BadArg) if the term is not a map or if
    /// key doesn't exist in the map.
    ///
    /// ### Erlang equivalent
    /// ```erlang
    /// maps:get(Key, Map)
    /// ```
    #[inline]
    pub fn map_get(self, key: impl Encoder) -> NifResult<Term<'a>> {
        let env = self.get_env();
        match unsafe {
            map::get_map_value(env.as_c_arg(), self.as_c_arg(), key.encode(env).as_c_arg())
        } {
            Some(value) => Ok(unsafe { Term::new(env, value) }),
            None => Err(Error::BadArg),
        }
    }

    /// Gets the size of a map term.
    ///
    /// Returns Err(Error::BadArg) if the term is not a map.
    ///
    /// ### Erlang equivalent
    /// ```erlang
    /// map_size(Map)
    /// ```
    #[inline]
    pub fn map_size(self) -> NifResult<usize> {
        let env = self.get_env();
        unsafe { map::get_map_size(env.as_c_arg(), self.as_c_arg()).ok_or(Error::BadArg) }
    }

    /// Makes a copy of the self map term and sets key to value.
    /// If the value already exists, it is overwritten.
    ///
    /// Returns Err(Error::BadArg) if the term is not a map.
    ///
    /// ### Erlang equivalent
    /// ```erlang
    /// maps:put(Key, Value, Map)
    /// ```
    #[inline]
    pub fn map_put(self, key: impl Encoder, value: impl Encoder) -> NifResult<Term<'a>> {
        let env = self.get_env();

        match unsafe {
            map::map_put(
                env.as_c_arg(),
                self.as_c_arg(),
                key.encode(env).as_c_arg(),
                value.encode(env).as_c_arg(),
            )
        } {
            Some(inner) => Ok(unsafe { Term::new(env, inner) }),
            None => Err(Error::BadArg),
        }
    }

    /// Makes a copy of the self map term and removes key. If the key
    /// doesn't exist, the original map is returned.
    ///
    /// Returns Err(Error::BadArg) if the term is not a map.
    ///
    /// ### Erlang equivalent
    /// ```erlang
    /// maps:remove(Key, Map)
    /// ```
    #[inline]
    pub fn map_remove(self, key: impl Encoder) -> NifResult<Term<'a>> {
        let env = self.get_env();

        match unsafe {
            map::map_remove(env.as_c_arg(), self.as_c_arg(), key.encode(env).as_c_arg())
        } {
            Some(inner) => Ok(unsafe { Term::new(env, inner) }),
            None => Err(Error::BadArg),
        }
    }

    /// Makes a copy of the self map term where key is set to value.
    ///
    /// Returns Err(Error::BadArg) if the term is not a map of if key
    /// doesn't exist.
    #[inline]
    pub fn map_update(self, key: impl Encoder, new_value: impl Encoder) -> NifResult<Term<'a>> {
        let env = self.get_env();

        match unsafe {
            map::map_update(
                env.as_c_arg(),
                self.as_c_arg(),
                key.encode(env).as_c_arg(),
                new_value.encode(env).as_c_arg(),
            )
        } {
            Some(inner) => Ok(unsafe { Term::new(env, inner) }),
            None => Err(Error::BadArg),
        }
    }
}

struct SimpleMapIterator<'a> {
    map: Term<'a>,
    entry: map::MapIteratorEntry,
    iter: Option<map::ErlNifMapIterator>,
    done: bool,
}

impl<'a> SimpleMapIterator<'a> {
    fn next(&mut self) -> Option<(Term<'a>, Term<'a>)> {
        if self.done {
            return None;
        }

        let iter = loop {
            match self.iter.as_mut() {
                None => {
                    match unsafe {
                        map::map_iterator_create(
                            self.map.get_env().as_c_arg(),
                            self.map.as_c_arg(),
                            self.entry,
                        )
                    } {
                        Some(iter) => {
                            self.iter = Some(iter);
                            continue;
                        }
                        None => {
                            self.done = true;
                            return None;
                        }
                    }
                }
                Some(iter) => {
                    break iter;
                }
            }
        };

        let env = self.map.get_env();

        unsafe {
            match map::map_iterator_get_pair(env.as_c_arg(), iter) {
                Some((key, value)) => {
                    match self.entry {
                        map::MapIteratorEntry::First => {
                            map::map_iterator_next(env.as_c_arg(), iter);
                        }
                        map::MapIteratorEntry::Last => {
                            map::map_iterator_prev(env.as_c_arg(), iter);
                        }
                    }
                    let key = Term::new(env, key);
                    Some((key, Term::new(env, value)))
                }
                None => {
                    self.done = true;
                    None
                }
            }
        }
    }
}

impl Drop for SimpleMapIterator<'_> {
    fn drop(&mut self) {
        if let Some(iter) = self.iter.as_mut() {
            unsafe {
                map::map_iterator_destroy(self.map.get_env().as_c_arg(), iter);
            }
        }
    }
}

pub struct MapIterator<'a> {
    forward: SimpleMapIterator<'a>,
    reverse: SimpleMapIterator<'a>,
    remaining: usize,
}

impl<'a> MapIterator<'a> {
        pub fn new(map: Term<'a>) -> Option<MapIterator<'a>> {
        if map.is_map() {
            let size = map.map_size().ok()?;
            Some(MapIterator {
                forward: SimpleMapIterator {
                    map,
                    entry: map::MapIteratorEntry::First,
                    iter: None,
                    done: false,
                },
                reverse: SimpleMapIterator {
                    map,
                    entry: map::MapIteratorEntry::Last,
                    iter: None,
                    done: false,
                },
                remaining: size,
            })
        } else {
            None
        }
    }
}

impl<'a> Iterator for MapIterator<'a> {
    type Item = (Term<'a>, Term<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.forward.next().map(|item| {
            self.remaining -= 1;
            item
        })
    }
}

impl DoubleEndedIterator for MapIterator<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.reverse.next().map(|item| {
            self.remaining -= 1;
            item
        })
    }
}

impl<'a> Decoder<'a> for MapIterator<'a> {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        match MapIterator::new(term) {
            Some(iter) => Ok(iter),
            None => Err(Error::BadArg),
        }
    }
}