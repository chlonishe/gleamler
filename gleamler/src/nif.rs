use crate::codegen_runtime::{NIF_ENV, NIF_TERM, c_char, c_int, c_uint};

pub struct Nif {
    pub name: *const c_char,
    pub arity: c_uint,
    pub flags: c_uint,
    pub raw_func:
        unsafe extern "C" fn(nif_env: NIF_ENV, argc: c_int, argv: *const NIF_TERM) -> NIF_TERM,
}
