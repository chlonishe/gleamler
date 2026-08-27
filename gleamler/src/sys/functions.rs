#![allow(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_os = "windows"))]
use super::nif_filler::DynNifFiller;
use super::types::*;

use std::sync::OnceLock;

static DYN_NIF_CALLBACKS: OnceLock<DynNifCallbacks> = OnceLock::new();

pub fn callbacks() -> &'static DynNifCallbacks {
    DYN_NIF_CALLBACKS
        .get()
        .expect("NIF callbacks not initialized")
}

pub unsafe fn internal_set_symbols(callbacks: DynNifCallbacks) {
    if DYN_NIF_CALLBACKS.set(callbacks).is_err() {
        panic!("gleamler: NIF callbacks already initialized");
    }
}

#[cfg(not(target_os = "windows"))]
pub fn internal_write_symbols() {
    DYN_NIF_CALLBACKS.get_or_init(|| {
        let mut callbacks = DynNifCallbacks::default();
        let filler = super::nif_filler::new();
        callbacks.write_symbols(filler);
        callbacks
    });
}

#[cfg(target_os = "windows")]
pub fn internal_write_symbols() {}

pub unsafe fn enif_make_pid(_env: *mut ErlNifEnv, pid: ErlNifPid) -> ERL_NIF_TERM {
    pid.pid
}

pub unsafe fn enif_compare_pids(pid1: *const ErlNifPid, pid2: *const ErlNifPid) -> c_int {
    enif_compare((*pid1).pid, (*pid2).pid)
}

include!(concat!(env!("OUT_DIR"), "/nif_api.snippet.rs"));
