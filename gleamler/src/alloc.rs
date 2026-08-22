use std::alloc::{GlobalAlloc, Layout, System};

use crate::sys::{c_void, enif_alloc, enif_free};

/// Allocator implementation that forwards all allocation calls to Erlang's allocator.
pub struct EnifAllocator;

// OTP's enif_alloc guarantees alignment suitable for any C variable, which formally means
// max_align_t. On Unix x86_64 this is 16 bytes, but on Windows x64 max_align_t is only 8.
// To avoid UB on layouts with >8 byte alignment, we conservatively route everything above
// pointer alignment through the system allocator
const ENIF_MAX_ALIGN: usize = std::mem::align_of::<usize>();

unsafe impl GlobalAlloc for EnifAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= ENIF_MAX_ALIGN {
            return unsafe { enif_alloc(layout.size()) as *mut u8 };
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.align() <= ENIF_MAX_ALIGN {
            unsafe { enif_free(ptr as *mut c_void) };
        } else {
            unsafe { System.dealloc(ptr, layout) };
        }
    }
}
