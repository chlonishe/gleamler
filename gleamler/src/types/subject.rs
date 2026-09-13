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

    /// Save the subject for sending messages from non-scheduler threads.
    pub fn save(&self, owned_env: &crate::OwnedEnv) -> SavedSubject<T> {
        SavedSubject {
            pid: self.pid,
            tag: owned_env.save(self.tag),
            _marker: PhantomData,
        }
    }

    /// Send a message to the subject from a non-scheduler thread using a saved tag.
    ///
    /// The tag term is loaded from `tag_env` and copied into an ephemeral message
    /// environment for transmission via `enif_send`, keeping `tag_env` (and `tag`)
    /// valid across multiple sends.
    pub fn send_from_owned(
        pid: &LocalPid,
        tag: &crate::env::SavedTerm,
        tag_env: &crate::OwnedEnv,
        message: T,
    ) -> Result<(), SendError>
    where
        T: Encoder,
    {
        let mut msg_env = crate::OwnedEnv::new();
        tag_env.run(|t_env| {
            let loaded_tag = tag.load(t_env);
            msg_env.send_and_clear(pid, |m_env| {
                (loaded_tag.in_env(m_env), message).encode(m_env)
            })
        })
    }
}

/// A subject saved into an [`OwnedEnv`] for sending messages from background threads.
#[derive(Clone)]
pub struct SavedSubject<T> {
    pid: LocalPid,
    tag: crate::env::SavedTerm,
    _marker: PhantomData<T>,
}

unsafe impl<T> Send for SavedSubject<T> {}

impl<T> SavedSubject<T> {
    pub fn pid(&self) -> LocalPid {
        self.pid
    }

    pub fn tag(&self) -> &crate::env::SavedTerm {
        &self.tag
    }

    pub fn send(&self, tag_env: &crate::OwnedEnv, message: T) -> Result<(), SendError>
    where
        T: Encoder,
    {
        Subject::send_from_owned(&self.pid, &self.tag, tag_env, message)
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
