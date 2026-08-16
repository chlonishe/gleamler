use std::alloc::{GlobalAlloc, Layout};

use crate::sys::{c_void, enif_alloc, enif_free};

/// Allocator implementation that forwards all allocation calls to Erlang's allocator. Allows the
/// memory usage to be tracked by the BEAM.
pub struct EnifAllocator;

unsafe impl GlobalAlloc for EnifAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let min_align = std::mem::align_of::<usize>();

        if layout.align() <= min_align {
            return unsafe { enif_alloc(layout.size()) as *mut u8 };
        }

        // Overallocate and store the original pointer in memory immediately before the aligned
        // section.
        //
        // The requested size is chosen such that we can always get an aligned buffer of size
        // `layout.size()`: Ignoring `SIZEOF_USIZE`, there must always be an aligned pointer in
        // the interval `[ptr, layout.align())`, so in the worst case, we have to pad with
        // `layout.align() - 1`. The requirement for an additional `usize` just shifts the
        // problem without changing the padding requirement.
        let total_size = std::mem::size_of::<usize>()
            .checked_add(layout.size())
            .and_then(|s| s.checked_add(layout.align() - 1))
            .expect("allocation size overflow");
        
        let ptr = unsafe { enif_alloc(total_size) as *mut u8 };

        if ptr.is_null() {
            return ptr;
        }
        // Shift the returned pointer to make space for the original pointer
        let ptr1 = ptr.wrapping_add(std::mem::size_of::<usize>());

        // Align the result to the requested alignment
        let aligned_ptr = ptr1.wrapping_add(ptr1.align_offset(layout.align()));
        
        // Write the original pointer immediately in front of the aligned pointer
        let header = aligned_ptr.wrapping_sub(std::mem::size_of::<usize>());
        unsafe { *(header as *mut usize) = ptr as usize };

        aligned_ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let min_align = std::mem::align_of::<usize>();

        let ptr = if layout.align() > min_align {
            // Retrieve the original pointer
            let header = ptr.wrapping_sub(std::mem::size_of::<usize>());
            let ptr = unsafe { *(header as *mut usize) };
            ptr as *mut c_void
        } else {
            ptr as *mut c_void
        };

        unsafe { enif_free(ptr) };
    }
}