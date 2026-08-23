//! Functions used by runtime generated code. Should not be used.

use std::ffi::{CStr, CString};
use std::fmt;
use std::cell::Cell;

use crate::schedule::SchedulerFlags;
use crate::types::atom;
use crate::{Encoder, Env, OwnedBinary, Term};

// Re-export of inventory
pub use inventory;

// Re-export of ctor for resource_impl macro
pub use ctor;

// Re-export of resource registration
pub use crate::resource::Registration as ResourceRegistration;

// Names used by the `gleamler::init!` macro or other generated code.
pub use crate::wrapper::exception::raise_exception;
pub use crate::wrapper::{
    DEF_NIF_ENTRY, DEF_NIF_FUNC, NIF_ENV, NIF_MAJOR_VERSION, NIF_MINOR_VERSION, NIF_TERM, c_char,
    c_int, c_uint, c_void, get_nif_resource_type_init_size,
};

pub use crate::sys::{DynNifCallbacks, internal_set_symbols, internal_write_symbols};

/// Auto-registration entry for `#[gleam_nif]` functions.
pub struct NifRegistration {
    pub nif: &'static crate::Nif,
}

unsafe impl Send for NifRegistration {}
unsafe impl Sync for NifRegistration {}

/// # Safety
pub unsafe trait NifReturnable {
    unsafe fn into_returned(self, env: Env) -> NifReturned;
}

thread_local! {
    pub static CURRENT_NIF_CONTINUATION: Cell<Option<(
        *const c_char,                                // static name (\0-terminated)
        unsafe extern "C" fn(NIF_ENV, i32, *const NIF_TERM) -> NIF_TERM,
        i32,                                          // argc
        *const NIF_TERM,                              // argv
    )>> = const { Cell::new(None) };
}

/// # Safety
/// `name` must point to a null-terminated string with static lifetime.
pub unsafe fn set_nif_continuation(
    name: *const c_char,
    fun: unsafe extern "C" fn(NIF_ENV, i32, *const NIF_TERM) -> NIF_TERM,
    argc: i32,
    argv: *const NIF_TERM,
) {
    CURRENT_NIF_CONTINUATION.with(|c| {
        c.set(Some((name, fun, argc, argv)));
    });
}

pub enum NifOutcome<T> {
    Done(T),
    Yield(SchedulerFlags),
}

unsafe impl<T> NifReturnable for NifOutcome<T>
where
    T: NifReturnable,
{
    unsafe fn into_returned(self, env: Env) -> NifReturned {
        match self {
            NifOutcome::Done(v) => unsafe { v.into_returned(env) },
            NifOutcome::Yield(flags) => {
                CURRENT_NIF_CONTINUATION.with(|c| {
                    let (name, fun, argc, argv) = c.get().expect(
                        "NifOutcome::Yield may only be used inside a #[gleam_nif] function"
                    );

                    // name создан макросом как статическая строка с \0 на конце
                    let cstr = unsafe { CStr::from_ptr(name) };
                    let fun_name = CString::from(cstr);

                    let args = unsafe {
                        std::slice::from_raw_parts(argv, argc as usize).to_vec()
                    };

                    NifReturned::Reschedule {
                        fun_name,
                        flags,
                        fun,
                        args,
                    }
                })
            }
        }
    }
}

unsafe impl<T> NifReturnable for T
where
    T: crate::Encoder + std::panic::RefUnwindSafe,
{
    unsafe fn into_returned(self, env: Env) -> NifReturned {
        if let Ok(res) = std::panic::catch_unwind(|| NifReturned::Term(self.encode(env).as_c_arg()))
        {
            res
        } else {
            let term = atom::nif_panicked().as_c_arg();
            NifReturned::Raise(term)
        }
    }
}

unsafe impl<T> NifReturnable for Result<T, crate::error::Error>
where
    T: NifReturnable,
{
    unsafe fn into_returned(self, env: Env) -> NifReturned {
        match self {
            Ok(inner) => unsafe { inner.into_returned(env) },
            Err(inner) => unsafe { inner.into_returned(env) },
        }
    }
}

unsafe impl NifReturnable for OwnedBinary {
    unsafe fn into_returned(self, env: Env) -> NifReturned {
        NifReturned::Term(self.release(env).encode(env).as_c_arg())
    }
}

pub enum NifReturned {
    Term(NIF_TERM),
    Raise(NIF_TERM),
    BadArg,
    Reschedule {
        fun_name: CString,
        flags: crate::schedule::SchedulerFlags,
        fun: unsafe extern "C" fn(NIF_ENV, i32, *const NIF_TERM) -> NIF_TERM,
        args: Vec<NIF_TERM>,
    },
}

impl NifReturned {
    pub unsafe fn apply(self, env: Env) -> NIF_TERM {
        match self {
            NifReturned::Term(inner) => inner,
            NifReturned::BadArg => unsafe {
                crate::wrapper::exception::raise_badarg(env.as_c_arg())
            },
            NifReturned::Raise(inner) => unsafe {
                crate::wrapper::exception::raise_exception(env.as_c_arg(), inner)
            },
            NifReturned::Reschedule {
                fun_name,
                flags,
                fun,
                args,
            } => unsafe {
                crate::sys::enif_schedule_nif(
                    env.as_c_arg(),
                    fun_name.as_ptr() as *const c_char,
                    flags as i32,
                    fun,
                    args.len() as i32,
                    args.as_ptr(),
                )
            },
        }
    }
}

impl fmt::Debug for NifReturned {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            NifReturned::BadArg => write!(fmt, "{{error, badarg}}"),
            NifReturned::Term(s) => write!(fmt, "{{ok, {s}}}"),
            NifReturned::Raise(s) => write!(fmt, "throw({s})"),
            NifReturned::Reschedule { .. } => write!(fmt, "reschedule()"),
        }
    }
}

pub unsafe fn handle_nif_init_call<'a>(
    function: for<'b> fn(Env<'b>, Term<'b>) -> bool,
    env: Env<'a>,
    load_info: Term<'a>,
) -> c_int {
    std::panic::catch_unwind(|| function(env, load_info)).map_or(1, |x| i32::from(!x))
}

pub fn handle_nif_result<T>(result: std::thread::Result<T>, env: Env) -> NifReturned
where
    T: NifReturnable,
{
    unsafe {
        match result {
            Ok(res) => NifReturnable::into_returned(res, env),
            Err(_) => {
                let term = atom::nif_panicked().as_c_arg();
                NifReturned::Raise(term)
            }
        }
    }
}

pub const fn min_erts() -> &'static [u8] {
    if cfg!(feature = "nif_version_2_18") {
        b"OTP-29.0\0"
    } else if cfg!(feature = "nif_version_2_17") {
        b"OTP-26.0\0"
    } else if cfg!(feature = "nif_version_2_16") {
        b"OTP-24.0\0"
    } else if cfg!(feature = "nif_version_2_15") {
        b"OTP-22.0\0"
    } else {
        b"OTP-21.0\0"
    }
}
