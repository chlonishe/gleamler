use crate::Env;
use crate::sys::enif_consume_timeslice;
use crate::wrapper::ErlNifTaskFlags;

pub enum SchedulerFlags {
    Normal = ErlNifTaskFlags::ERL_NIF_NORMAL_JOB as isize,
    DirtyCpu = ErlNifTaskFlags::ERL_NIF_DIRTY_JOB_CPU_BOUND as isize,
    DirtyIo = ErlNifTaskFlags::ERL_NIF_DIRTY_JOB_IO_BOUND as isize,
}

pub fn consume_timeslice(env: Env, percent: i32) -> bool {
    let success = unsafe { enif_consume_timeslice(env.as_c_arg(), percent) };
    success == 1
}

use crate::sys::{
    ErlNifSelectFlags, ERL_NIF_SELECT_ERROR_CANCELLED, ERL_NIF_SELECT_FAILED,
    ERL_NIF_SELECT_NOTSUP, ERL_NIF_SELECT_READ, ERL_NIF_SELECT_READ_CANCELLED,
    ERL_NIF_SELECT_STOP, ERL_NIF_SELECT_WRITE, ERL_NIF_SELECT_WRITE_CANCELLED,
};

/// Flags for [`ResourceArc::select`](crate::ResourceArc::select)
///
/// Available since NIF version 2.12 (OTP 22)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectFlags(pub(crate) ErlNifSelectFlags);

impl SelectFlags {
    pub const READ: Self = Self(ERL_NIF_SELECT_READ);
    pub const WRITE: Self = Self(ERL_NIF_SELECT_WRITE);
    pub const STOP: Self = Self(ERL_NIF_SELECT_STOP);
    pub const FAILED: Self = Self(ERL_NIF_SELECT_FAILED);
    pub const READ_CANCELLED: Self = Self(ERL_NIF_SELECT_READ_CANCELLED);
    pub const WRITE_CANCELLED: Self = Self(ERL_NIF_SELECT_WRITE_CANCELLED);
    pub const ERROR_CANCELLED: Self = Self(ERL_NIF_SELECT_ERROR_CANCELLED);
    pub const NOTSUP: Self = Self(ERL_NIF_SELECT_NOTSUP);

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub(crate) fn as_c_int(self) -> crate::sys::c_int {
        self.0
    }
}