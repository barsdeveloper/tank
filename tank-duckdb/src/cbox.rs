use libduckdb_sys::duckdb_result;
use std::{
    ops::{Deref, DerefMut},
    sync::atomic::AtomicPtr,
};

pub(crate) trait NullCheck {
    fn is_null(&self) -> bool;
}

impl<T> NullCheck for *const T {
    fn is_null(&self) -> bool {
        (*self as *const T).is_null()
    }
}

impl<T> NullCheck for *mut T {
    fn is_null(&self) -> bool {
        (*self as *const T).is_null()
    }
}

impl<T> NullCheck for AtomicPtr<T> {
    fn is_null(&self) -> bool {
        self.load(std::sync::atomic::Ordering::Relaxed).is_null()
    }
}

impl<T: NullCheck> NullCheck for &T {
    fn is_null(&self) -> bool {
        (*self).is_null()
    }
}

impl<T: NullCheck> NullCheck for &mut T {
    fn is_null(&self) -> bool {
        false
    }
}

impl NullCheck for duckdb_result {
    fn is_null(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub(crate) struct CBox<T: NullCheck> {
    pub(crate) ptr: T,
    dealloc: fn(T),
}

impl<T: NullCheck> CBox<T> {
    pub fn new(ptr: T, dealloc: fn(T)) -> Self {
        Self { ptr, dealloc }
    }
}

impl<T: NullCheck> Drop for CBox<T> {
    fn drop(&mut self) {
        if !self.is_null() {
            unsafe {
                (self.dealloc)(std::ptr::read(&self.ptr as *const T));
            }
        }
    }
}

impl<T: NullCheck> Deref for CBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

impl<T: NullCheck> DerefMut for CBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ptr
    }
}

unsafe impl<T: NullCheck> Send for CBox<T> {}
unsafe impl<T: NullCheck> Sync for CBox<T> {}

#[cfg(test)]
mod tests {
    use std::ptr;

    use crate::cbox::CBox;

    #[tokio::test]
    async fn cbox_raw_pointer() {
        static mut DESTROYED: bool = false;
        let v = 123;
        let ptr: *const i32 = &v;
        unsafe {
            {
                let ptr = CBox::new(ptr::null::<*const i32>(), |_| DESTROYED = true);
                assert_eq!(*ptr, ptr::null());
            }
            assert!(!DESTROYED);
            {
                let ptr = CBox::new(ptr, |_| DESTROYED = true);
                assert_eq!(**ptr, 123);
                assert!(!DESTROYED);
            }
            assert!(DESTROYED)
        }
    }
}
