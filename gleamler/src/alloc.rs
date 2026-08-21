use std::alloc::{GlobalAlloc, Layout, System};

use crate::sys::{c_void, enif_alloc, enif_free};

/// Allocator implementation that forwards all allocation calls to Erlang's allocator. Allows the
/// memory usage to be tracked by the BEAM.
pub struct EnifAllocator;

// On x86_64 BEAM's enif_alloc aligns to max_align_t (16 bytes),
// so we can safely route SIMD-friendly layouts (up to __m128) through it.
// On other architectures we stay conservative and stick to pointer alignment
#[cfg(target_arch = "x86_64")]
const ENIF_MAX_ALIGN: usize = std::mem::align_of::<std::arch::x86_64::__m128>();

#[cfg(not(target_arch = "x86_64"))]
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