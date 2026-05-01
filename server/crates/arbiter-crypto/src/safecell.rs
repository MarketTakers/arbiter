use memsafe::MemSafe;
use std::{
    any::type_name,
    fmt,
    ops::{Deref, DerefMut},
};

pub trait SafeCellHandle<T> {
    type CellRead<'a>: Deref<Target = T>
    where
        Self: 'a,
        T: 'a;
    type CellWrite<'a>: Deref<Target = T> + DerefMut<Target = T>
    where
        Self: 'a,
        T: 'a;

    fn new(value: T) -> Self
    where
        Self: Sized;

    fn read(&mut self) -> Self::CellRead<'_>;
    fn write(&mut self) -> Self::CellWrite<'_>;

    fn new_inline_default<F>(f: F) -> Self
    where
        Self: Sized,
        T: Default,
        F: for<'a> FnOnce(&'a mut T),
    {
        let mut cell = Self::new(T::default());
        {
            let mut handle = cell.write();
            f(&mut *handle);
        }
        cell
    }

    fn new_inline<F>(f: Box<F>) -> Self
    where
        Self: Sized,
        F: for<'a> FnOnce() -> T,
    {
        Self::new(f())
    }

    #[inline(always)]
    fn read_inline<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&*self.read())
    }

    #[inline(always)]
    fn write_inline<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut *self.write())
    }
}

pub struct MemSafeCell<T>(MemSafe<T>);

impl<T> fmt::Debug for MemSafeCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemSafeCell")
            .field("inner", &format_args!("<protected {}>", type_name::<T>()))
            .finish()
    }
}

impl<T> SafeCellHandle<T> for MemSafeCell<T> {
    type CellRead<'a>
        = memsafe::MemSafeRead<'a, T>
    where
        Self: 'a,
        T: 'a;
    type CellWrite<'a>
        = memsafe::MemSafeWrite<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn new(value: T) -> Self {
        match MemSafe::new(value) {
            Ok(inner) => Self(inner),
            Err(err) => {
                // If protected memory cannot be allocated, process integrity is compromised.
                abort_memory_breach("safe cell allocation", &err)
            }
        }
    }

    #[inline(always)]
    fn read(&mut self) -> Self::CellRead<'_> {
        match self.0.read() {
            Ok(inner) => inner,
            Err(err) => abort_memory_breach("safe cell read", &err),
        }
    }

    #[inline(always)]
    fn write(&mut self) -> Self::CellWrite<'_> {
        match self.0.write() {
            Ok(inner) => inner,
            Err(err) => {
                // If protected memory becomes unwritable here, treat it as a fatal memory breach.
                abort_memory_breach("safe cell write", &err)
            }
        }
    }
}

fn abort_memory_breach(action: &str, err: &memsafe::error::MemoryError) -> ! {
    eprintln!("fatal {action}: {err}");
    // SAFETY: Intentionally cause a segmentation fault to prevent further execution in a compromised state.
    unsafe {
        let unsafe_pointer = std::ptr::null_mut::<u8>();
        std::ptr::write_volatile(unsafe_pointer, 0);
    }
    std::process::abort();
}

pub type SafeCell<T> = MemSafeCell<T>;
