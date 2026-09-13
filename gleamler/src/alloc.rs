use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr;

use crate::sys::{c_void, enif_alloc, enif_free, enif_realloc};

/// Allocator implementation that forwards all allocation calls to Erlang's allocator.
pub struct EnifAllocator;

// OTP's enif_alloc guarantees alignment suitable for any C variable, which formally means
// `align_of::<max_align_t>()`. On 64-bit Unix (x86_64 and aarch64), this is 16 bytes.
const ENIF_MAX_ALIGN: usize = if cfg!(any(
    all(unix, any(target_arch = "x86_64", target_arch = "aarch64")),
    all(windows, target_arch = "x86_64")
)) {
    16
} else {
    std::mem::align_of::<usize>()
};

unsafe impl GlobalAlloc for EnifAllocator {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            return layout.align() as *mut u8;
        }
        if layout.align() <= ENIF_MAX_ALIGN {
            unsafe { enif_alloc(layout.size()) as *mut u8 }
        } else {
            unsafe { System.alloc(layout) }
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() == 0 {
            return;
        }
        if layout.align() <= ENIF_MAX_ALIGN {
            unsafe { enif_free(ptr as *mut c_void) };
        } else {
            unsafe { System.dealloc(ptr, layout) };
        }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.size() == 0 {
            return unsafe {
                self.alloc(Layout::from_size_align_unchecked(new_size, layout.align()))
            };
        }
        if new_size == 0 {
            unsafe { self.dealloc(ptr, layout) };
            return layout.align() as *mut u8;
        }
        if layout.align() <= ENIF_MAX_ALIGN {
            unsafe { enif_realloc(ptr as *mut c_void, new_size) as *mut u8 }
        } else {
            unsafe { System.realloc(ptr, layout, new_size) }
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            return layout.align() as *mut u8;
        }
        if layout.align() <= ENIF_MAX_ALIGN {
            let ptr = unsafe { enif_alloc(layout.size()) };
            if !ptr.is_null() {
                unsafe { ptr::write_bytes(ptr, 0, layout.size()) };
            }
            ptr as *mut u8
        } else {
            unsafe { System.alloc_zeroed(layout) }
        }
    }
}
