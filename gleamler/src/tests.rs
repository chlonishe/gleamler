#[cfg(test)]
use crate::codegen_runtime::min_erts;
use crate::resource::util::{align_alloced_mem_for_struct, get_alloc_size_struct};
use std::mem;

#[test]
fn min_erts_matches_feature() {
    let expected = if cfg!(feature = "nif_version_2_18") {
        b"OTP-29"
    } else if cfg!(feature = "nif_version_2_17") {
        b"OTP-26"
    } else if cfg!(feature = "nif_version_2_16") {
        b"OTP-24"
    } else if cfg!(feature = "nif_version_2_15") {
        b"OTP-22"
    } else {
        b"OTP-21"
    };
    let erts = min_erts();
    assert!(
        erts.starts_with(expected),
        "expected {:?}, got {:?}",
        std::str::from_utf8(expected),
        std::str::from_utf8(erts)
    );
}

#[test]
fn alloc_size_u64() {
    assert_eq!(
        get_alloc_size_struct::<u64>(),
        mem::size_of::<u64>() + mem::align_of::<u64>()
    );
}

#[test]
fn alloc_size_struct_with_padding() {
    #[repr(C)]
    struct Padded {
        _a: u8,
        _b: u64,
    }
    assert_eq!(
        get_alloc_size_struct::<Padded>(),
        mem::size_of::<Padded>() + mem::align_of::<Padded>()
    );
}

#[test]
fn align_memory_for_struct() {
    let align = mem::align_of::<u64>();
    let base: usize = 0x1000;
    for offset in 0..align * 3 {
        let ptr = (base + offset) as *const std::ffi::c_void;
        let aligned = unsafe { align_alloced_mem_for_struct::<u64>(ptr) };
        let addr = aligned as usize;
        assert!(addr >= base + offset);
        assert!(addr < base + offset + align);
        assert_eq!(addr % align, 0, "alignment failed for offset {}", offset);
    }
}

#[test]
fn error_debug_formatting() {
    use crate::Error;
    let e = Error::BadArg;
    assert_eq!(format!("{:?}", e), "{error, badarg}");
}
