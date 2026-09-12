use crate::env::SendError;
use crate::{Decoder, Encoder, Env, Error, LocalPid, NifResult, Term};
use std::fmt;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct Subject<'a, T> {
    pid: LocalPid,
    tag: Term<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> fmt::Debug for Subject<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Subject")
            .field("pid", &self.pid)
            .field("tag", &self.tag)
            .finish()
    }
}

impl<'a, T> Subject<'a, T> {
    pub fn new(pid: LocalPid, tag: Term<'a>) -> Self {
        Self {
            pid,
            tag,
            _marker: PhantomData,
        }
    }

    pub fn pid(&self) -> LocalPid {
        self.pid
    }

    pub fn tag(&self) -> Term<'a> {
        self.tag
    }

    pub fn send(&self, env: Env<'a>, message: T) -> Result<(), SendError>
    where
        T: Encoder,
    {
        let payload = (self.tag, message);
        env.send(&self.pid, payload)
    }
}

impl<'a, T: 'a> Decoder<'a> for Subject<'a, T> {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let (tag_atom, pid, tag): (crate::Atom, LocalPid, Term<'a>) = term.decode()?;
        if let Ok(s) = tag_atom.to_term(term.get_env()).atom_to_string()
            && s == "subject"
        {
            return Ok(Subject {
                pid,
                tag,
                _marker: PhantomData,
            });
        }
        Err(Error::BadArg)
    }
}

impl<'a, T> Encoder for Subject<'a, T> {
    fn encode<'b>(&self, env: Env<'b>) -> Term<'b> {
        let subject_atom = crate::types::atom::Atom::from_str(env, "subject").unwrap();
        (subject_atom, self.pid, self.tag.in_env(env)).encode(env)
    }
}
