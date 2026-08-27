#[cfg(not(target_os = "windows"))]
pub(crate) trait DynNifFiller {
    fn write<T: Copy>(&self, field: &mut Option<T>, name: &str);
}

#[cfg(not(target_os = "windows"))]
mod internal {
    use super::DynNifFiller;
    use libc::{RTLD_GLOBAL, RTLD_NOLOAD, RTLD_NOW};
    use libloading::os::unix::Library;

    const FLAGS: i32 = RTLD_GLOBAL | RTLD_NOLOAD | RTLD_NOW;
    const BEAM_LOC: &str = "GLEAMLER_BEAM_LIBRARY_PATH";

    pub(crate) struct DlsymNifFiller {
        lib: libloading::Library,
    }

    impl DlsymNifFiller {
        pub(crate) fn new() -> Self {
            let beam_location = match std::env::var(BEAM_LOC) {
                Ok(val) if !val.is_empty() => Some(val),
                _ => None,
            };
            let beam_path = beam_location.as_deref().map(std::ffi::OsStr::new);
            let lib = unsafe { Library::open(beam_path, FLAGS) }
                .expect(
                    "gleamler: failed to open BEAM VM library for NIF symbol resolution. \
                     Ensure GLEAMLER_BEAM_LIBRARY_PATH is correct and BEAM was loaded with RTLD_GLOBAL",
                );
            DlsymNifFiller { lib: lib.into() }
        }
    }

    impl DynNifFiller for DlsymNifFiller {
        fn write<T: Copy>(&self, field: &mut Option<T>, name: &str) {
            let symbol = unsafe { self.lib.get::<T>(name.as_bytes()) }
                .unwrap_or_else(|e| panic!("gleamler: NIF symbol `{name}` not found: {e}"));
            *field = Some(*symbol);
        }
    }

    pub(crate) fn new() -> impl DynNifFiller {
        DlsymNifFiller::new()
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) use internal::new;
