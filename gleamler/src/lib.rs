#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_uint, c_void};

pub type ErlNifEnv = c_void;
pub type ErlNifTerm = usize;

#[repr(C)]
pub struct ErlNifFunc {
    pub name: *const c_char,
    pub arity: c_uint,
    pub fptr: unsafe extern "C" fn(*mut ErlNifEnv, c_int, *const ErlNifTerm) -> ErlNifTerm,
    pub flags: c_uint,
}

#[repr(C)]
pub struct ErlNifEntry {
    pub major: c_int,
    pub minor: c_int,
    pub name: *const c_char,
    pub num_of_funcs: c_int,
    pub funcs: *const ErlNifFunc,
    pub load: Option<unsafe extern "C" fn(*mut ErlNifEnv, *mut c_void, c_int) -> c_int>,
    pub reload: Option<unsafe extern "C" fn(*mut ErlNifEnv, *mut c_void, *mut c_void) -> c_int>,
    pub upgrade: Option<unsafe extern "C" fn(*mut ErlNifEnv, *mut c_void, *mut c_void, *mut c_void) -> c_int>,
    pub unload: Option<unsafe extern "C" fn(*mut ErlNifEnv, *mut c_void)>,
    pub vm_variant: *const c_char,
    pub options: c_uint,
    pub sizeof_ErlNifEntry: usize,
}

pub struct UnsafeSyncEntry(pub ErlNifEntry);
unsafe impl Sync for UnsafeSyncEntry {}

pub struct UnsafeSyncFuncs(pub [ErlNifFunc; 1]);
unsafe impl Sync for UnsafeSyncFuncs {}

pub trait Decoder {
    fn decode(term: ErlNifTerm) -> Self;
}

pub trait Encoder {
    fn encode(self) -> ErlNifTerm;
}

impl Decoder for i64 {
    fn decode(term: ErlNifTerm) -> Self {
        (term >> 4) as i64
    }
}

impl Encoder for i64 {
    fn encode(self) -> ErlNifTerm {
        assert!(
            self >= -(1i64 << 59) && self < (1i64 << 59),
            "integer too large for small int encoding"
        );
        ((self as usize) << 4) | 0xF
    }
}

#[gleamler_macros::gleam_nif]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

static FUNCTIONS: UnsafeSyncFuncs = UnsafeSyncFuncs([ErlNifFunc {
    name: b"add\0".as_ptr() as *const c_char,
    arity: 2,
    fptr: ffi_add,
    flags: 0,
}]);

static ENTRY: UnsafeSyncEntry = UnsafeSyncEntry(ErlNifEntry {
    major: 2,
    minor: 17,
    name: b"gleamler_nif\0".as_ptr() as *const c_char,
    num_of_funcs: 1,
    funcs: &FUNCTIONS.0 as *const ErlNifFunc,
    load: None,
    reload: None,
    upgrade: None,
    unload: None,
    vm_variant: b"beam.vanilla\0".as_ptr() as *const c_char,
    options: 0,
    sizeof_ErlNifEntry: std::mem::size_of::<ErlNifEntry>(),
});

#[unsafe(no_mangle)]
pub extern "C" fn nif_init() -> *const ErlNifEntry {
    &ENTRY.0 as *const _
}