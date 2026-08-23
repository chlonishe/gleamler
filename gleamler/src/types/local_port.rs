use crate::sys::{ErlNifPort, enif_compare, enif_get_local_port, enif_is_port_alive};
use crate::{Decoder, Encoder, Env, Error, NifResult, Term};
use std::cmp::Ordering;
use std::mem::MaybeUninit;

/// A handle to an Erlang port.
#[derive(Copy, Clone, Debug)]
pub struct LocalPort {
    pub(crate) c: ErlNifPort,
}

impl LocalPort {
    #[inline]
    pub fn as_c_arg(&self) -> &ErlNifPort {
        &self.c
    }

    #[inline]
    pub fn from_c_arg(erl_nif_port: ErlNifPort) -> Self {
        LocalPort { c: erl_nif_port }
    }

    /// Check whether the given port is alive.
    pub fn is_alive(self, env: Env) -> bool {
        let res = unsafe { enif_is_port_alive(env.as_c_arg(), self.as_c_arg()) };
        res != 0
    }
}

impl<'a> Decoder<'a> for LocalPort {
    #[inline]
    fn decode(term: Term<'a>) -> NifResult<LocalPort> {
        let mut port = MaybeUninit::uninit();
        if unsafe {
            enif_get_local_port(
                term.get_env().as_c_arg(),
                term.as_c_arg(),
                port.as_mut_ptr(),
            )
        } == 0
        {
            return Err(Error::BadArg);
        }
        Ok(LocalPort {
            c: unsafe { port.assume_init() },
        })
    }
}

impl Encoder for LocalPort {
    #[inline]
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        unsafe { Term::new(env, self.c.port_id) }
    }
}

impl PartialEq for LocalPort {
    fn eq(&self, other: &Self) -> bool {
        unsafe { enif_compare(self.c.port_id, other.c.port_id) == 0 }
    }
}

impl Eq for LocalPort {}

impl PartialOrd for LocalPort {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LocalPort {
    fn cmp(&self, other: &Self) -> Ordering {
        let cmp = unsafe { enif_compare(self.c.port_id, other.c.port_id) };
        cmp.cmp(&0)
    }
}
