use std::alloc::{GlobalAlloc, Layout, System};

use crate::sys::{c_void, enif_alloc, enif_free};

/// Allocator implementation that forwards all allocation calls to Erlang's allocator. Allows the
/// memory usage to be tracked by the BEAM.
pub struct EnifAllocator;

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