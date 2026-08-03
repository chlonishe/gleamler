#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_os = "windows"))]
use super::nif_filler::DynNifFiller;
use super::types::*;

static mut DYN_NIF_CALLBACKS: DynNifCallbacks =
    unsafe { std::mem::MaybeUninit::zeroed().assume_init() };

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
#[used]
pub static mut TWinDynNifCallbacks: DynNifCallbacks =
    unsafe { std::mem::MaybeUninit::zeroed().assume_init() };

pub unsafe fn internal_set_symbols(callbacks: DynNifCallbacks) {
    DYN_NIF_CALLBACKS = callbacks;
}

#[cfg(not(target_os = "windows"))]
pub unsafe fn internal_write_symbols() {
    let filler = nif_filler::new();
    DYN_NIF_CALLBACKS.write_symbols(filler);
}

#[cfg(target_os = "windows")]
pub unsafe fn internal_write_symbols() {}

pub unsafe fn enif_make_pid(_env: *mut ErlNifEnv, pid: ErlNifPid) -> ERL_NIF_TERM {
    pid.pid
}

pub unsafe fn enif_compare_pids(pid1: *const ErlNifPid, pid2: *const ErlNifPid) -> c_int {
    enif_compare((*pid1).pid, (*pid2).pid)
}

include!(concat!(env!("OUT_DIR"), "/nif_api.snippet.rs"));