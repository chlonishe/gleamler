use crate::wrapper::{ErlNifPid, NIF_ENV, NIF_TERM};
use std::mem::MaybeUninit;

use crate::sys::{enif_get_local_pid, enif_make_pid};

pub unsafe fn get_local_pid(env: NIF_ENV, term: NIF_TERM) -> Option<ErlNifPid> {
    let mut pid = MaybeUninit::uninit();
    if unsafe { enif_get_local_pid(env, term, pid.as_mut_ptr()) } == 0 {
        return None;
    }
    Some(unsafe { pid.assume_init() })
}

pub unsafe fn make_pid(env: NIF_ENV, pid: ErlNifPid) -> NIF_TERM {
    unsafe { enif_make_pid(env, pid) }
}
