use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ptr;

use crate::sys::{
    c_void, enif_alloc_resource, enif_demonitor_process, enif_keep_resource, enif_make_resource,
    enif_make_resource_binary, enif_monitor_process, enif_release_resource,
};

use crate::{Binary, Decoder, Encoder, Env, Error, LocalPid, Monitor, NifResult, Term};

use super::traits::{Resource, ResourceExt};
use super::util::{align_alloced_mem_for_struct, get_alloc_size_struct};

/// A reference to a resource of type `T`.
///
/// This type is like `std::sync::Arc`: it provides thread-safe, reference-counted storage for Rust
/// data that can be shared across threads. Data stored this way is immutable by default. If you
/// need to modify data in a resource, use a `std::sync::Mutex` or `RwLock`.
///
/// Rust code and Erlang code can both have references to the same resource at the same time.  Rust
/// code uses `ResourceArc`; in Erlang, a reference to a resource is a kind of term.  You can
/// convert back and forth between the two using `Encoder` and `Decoder`.
pub struct ResourceArc<T>
where
    T: Resource,
{
    raw: *const c_void,
    inner: *mut T,
}

// Safe because T is `Sync` and `Send`.
unsafe impl<T> Send for ResourceArc<T> where T: Resource {}
unsafe impl<T> Sync for ResourceArc<T> where T: Resource {}

impl<T> std::panic::UnwindSafe for ResourceArc<T> where T: Resource {}
impl<T> std::panic::RefUnwindSafe for ResourceArc<T> where T: Resource {}

impl<T> ResourceArc<T>
where
    T: Resource,
{
    /// Makes a new ResourceArc from the given type. Note that the type must have Resource
    /// implemented for it. See module documentation for info on this.
    pub fn new(data: T) -> Self {
        let alloc_size = get_alloc_size_struct::<T>();
        let resource_type = T::get_resource_type().unwrap_or_else(|| {
            panic!(
                "Resource type `{}` has not been registered. Register it during `on_load` via `env.register::<{}>()`.",
                std::any::type_name::<T>(),
                std::any::type_name::<T>()
            );
        });
        let mem_raw = unsafe { enif_alloc_resource(resource_type, alloc_size) };
        if mem_raw.is_null() {
            panic!("enif_alloc_resource returned null (out of memory)");
        }
        let aligned_mem = unsafe { align_alloced_mem_for_struct::<T>(mem_raw) as *mut T };

        unsafe { ptr::write(aligned_mem, data) };

        ResourceArc {
            raw: mem_raw,
            inner: aligned_mem,
        }
    }

    /// Make a resource binary associated with the given resource
    ///
    /// The closure `f` is called with the referenced object and must return a slice with the same
    /// lifetime as the object. This means that the slice either has to be derived directly from
    /// the instance or that it has to have static lifetime.
    pub fn make_binary<'env, 'a, F>(&self, env: Env<'env>, f: F) -> Binary<'env>
    where
        F: FnOnce(&'a T) -> &'a [u8],
    {
        // This call is safe because `f` can only return a slice that lives at least as long as
        // the given instance of `T`.
        unsafe { self.make_binary_unsafe(env, f) }
    }

    /// Make a resource binary without strict lifetime checking
    ///
    /// The user *must* ensure that the lifetime of the returned slice is at least as long as the
    /// lifetime of the referenced instance.
    ///
    /// # Safety
    ///
    /// This function is only safe if the slice that is returned from the closure is guaranteed to
    /// live at least as long as the `ResourceArc` instance. If in doubt, use the safe version
    /// `ResourceArc::make_binary` which enforces this bound through its signature.
    pub unsafe fn make_binary_unsafe<'env, 'a, 'b, F>(&self, env: Env<'env>, f: F) -> Binary<'env>
    where
        F: FnOnce(&'a T) -> &'b [u8],
    {
        let bin = f(unsafe { &*self.inner });
        let binary = unsafe {
            enif_make_resource_binary(
                env.as_c_arg(),
                self.raw,
                bin.as_ptr() as *const c_void,
                bin.len(),
            )
        };

        let term = unsafe { Term::new(env, binary) };
        unsafe { Binary::from_term_and_slice(term, bin) }
    }

    fn from_term(term: Term) -> Result<Self, Error> {
        let (raw, inner) = unsafe { term.try_get_resource_ptrs::<T>() }.ok_or(Error::BadArg)?;
        unsafe { enif_keep_resource(raw) };
        Ok(ResourceArc { raw, inner })
    }

    fn as_term<'a>(&self, env: Env<'a>) -> Term<'a> {
        unsafe { Term::new(env, enif_make_resource(env.as_c_arg(), self.raw)) }
    }

    /// Return a pointer to the memory area allocated by the erlang VM.
    ///
    /// Note that this pointer does not necessarily point to the contained type but is commonly used
    /// as an object identifier for an allocated resource when interacting with low-level VM functions
    /// like [`enif_select()`](crate::sys::enif_select).
    pub fn as_c_arg(&self) -> *const c_void {
        self.raw
    }

    fn inner(&self) -> &T {
        unsafe { &*self.inner }
    }

    /// Start monitoring a process from a process-bound environment.
    ///
    /// # Panics
    ///
    /// Panics if `env` is not a process-bound environment (e.g. an `OwnedEnv`)
    pub fn monitor(&self, env: Env, pid: &LocalPid) -> Option<Monitor> {
        if !T::IMPLEMENTS_DOWN {
            panic!(
                "cannot monitor a resource of type `{}` because it does not set `IMPLEMENTS_DOWN = true`",
                std::any::type_name::<T>()
            );
        }

        // This panics if `env` is process-independent, which is exactly what we want:
        // enif_monitor_process requires a process-bound env
        env.pid();

        let mut mon = MaybeUninit::uninit();
        let res = unsafe {
            enif_monitor_process(env.as_c_arg(), self.raw, pid.as_c_arg(), mon.as_mut_ptr()) == 0
        };
        if res {
            Some(unsafe { Monitor::new(mon.assume_init()) })
        } else {
            None
        }
    }

    /// Stop monitoring a process from a process-bound environment
    ///
    /// # Panics
    ///
    /// Panics if `env` is not a process-bound environment (e.g. an `OwnedEnv`)
    pub fn demonitor(&self, env: Env, mon: &Monitor) -> bool {
        if !T::IMPLEMENTS_DOWN {
            panic!(
                "cannot demonitor a resource of type `{}` because it does not set `IMPLEMENTS_DOWN = true`",
                std::any::type_name::<T>()
            );
        }

        env.pid();

        unsafe { enif_demonitor_process(env.as_c_arg(), self.raw, mon.as_c_arg()) == 0 }
    }

    /// Monitor an OS event (file descriptor / handle) associated with this resource.
    ///
    /// Available since NIF version 2.12 (OTP 22).
    /// See [`enif_select`](https://www.erlang.org/doc/man/erl_nif.html#enif_select).
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // ErlNifEvent is an opaque handle
    pub fn select(
        &self,
        env: Env,
        event: crate::sys::ErlNifEvent,
        flags: crate::schedule::SelectFlags,
        pid: &crate::LocalPid,
        reference: Term,
    ) -> Result<(), Error> {
        let res = unsafe {
            crate::sys::enif_select(
                env.as_c_arg(),
                event,
                flags.as_c_int(),
                self.raw,
                pid.as_c_arg(),
                reference.as_c_arg(),
            )
        };
        if res == 0 { Ok(()) } else { Err(Error::BadArg) }
    }
}

impl<'a> Env<'a> {
    pub fn monitor<T: Resource>(
        &self,
        resource: &ResourceArc<T>,
        pid: &LocalPid,
    ) -> Option<Monitor> {
        resource.monitor(*self, pid)
    }

    pub fn demonitor<T: Resource>(&self, resource: &ResourceArc<T>, mon: &Monitor) -> bool {
        resource.demonitor(*self, mon)
    }

    /// # Safety
    #[cfg(feature = "nif_version_2_16")]
    pub unsafe fn dynamic_resource_call(
        self,
        module: crate::Atom,
        name: crate::Atom,
        resource: Term<'a>,
        call_data: *mut c_void,
    ) -> Result<(), super::DynamicResourceCallError> {
        use crate::sys::enif_dynamic_resource_call;

        let res = unsafe {
            enif_dynamic_resource_call(
                self.as_c_arg(),
                module.as_c_arg(),
                name.as_c_arg(),
                resource.as_c_arg(),
                call_data,
            )
        };

        if res == 0 {
            Ok(())
        } else {
            Err(super::DynamicResourceCallError)
        }
    }
}

impl<T> Deref for ResourceArc<T>
where
    T: Resource,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.inner()
    }
}

impl<T> Clone for ResourceArc<T>
where
    T: Resource,
{
    /// Cloning a `ResourceArc` simply increments the reference count for the
    /// resource. The `T` value is not cloned.
    fn clone(&self) -> Self {
        unsafe { enif_keep_resource(self.raw) };
        ResourceArc {
            raw: self.raw,
            inner: self.inner,
        }
    }
}

impl<T> Drop for ResourceArc<T>
where
    T: Resource,
{
    /// When a `ResourceArc` is dropped, the reference count is decremented. If
    /// there are no other references to the resource, the `T` value is dropped.
    ///
    /// However, note that in general, the Rust value in a resource is dropped
    /// at an unpredictable time: whenever the VM decides to do garbage
    /// collection.
    fn drop(&mut self) {
        unsafe { enif_release_resource(self.as_c_arg()) };
    }
}

impl<T: Resource> From<T> for ResourceArc<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Encoder for ResourceArc<T>
where
    T: Resource,
{
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        self.as_term(env)
    }
}
impl<'a, T> Decoder<'a> for ResourceArc<T>
where
    T: Resource + 'a,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        ResourceArc::from_term(term)
    }
}
