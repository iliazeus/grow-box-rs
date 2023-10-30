use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::fmt::{self, Debug, Formatter};
use std::mem::{forget, size_of, transmute, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::drop_in_place;

fn min_common_layout(l1: Layout, l2: Layout) -> Layout {
    if l1.size() >= l2.size() && l1.align() >= l2.align() {
        l1
    } else if l2.size() >= l1.size() && l2.align() >= l1.align() {
        l2
    } else {
        let max_size = usize::max(l1.size(), l2.size());
        let max_align = usize::max(l1.align(), l2.align());
        unsafe { Layout::from_size_align_unchecked(max_size, max_align) }
    }
}

pub struct GrowBox<T> {
    ptr: *mut T,
    layout: Layout,
}

impl<T> GrowBox<T> {
    #[inline(always)]
    unsafe fn uninit() -> Self {
        let layout = Layout::new::<T>();
        let ptr = alloc(layout);
        if !ptr.is_null() {
            Self {
                ptr: ptr.cast(),
                layout,
            }
        } else {
            handle_alloc_error(layout)
        }
    }

    #[inline(always)]
    unsafe fn set_layout(&mut self, layout: Layout) {
        dealloc(self.ptr.cast(), self.layout);
        let ptr = alloc(layout);
        if !ptr.is_null() {
            self.ptr = ptr.cast();
            self.layout = layout;
        } else {
            handle_alloc_error(layout)
        }
    }

    #[inline(always)]
    unsafe fn copy_bytes_from(&mut self, src: *const T) {
        let src: *const u8 = src.cast();
        let dst: *mut u8 = self.ptr.cast();
        for i in 0..size_of::<T>() {
            *dst.add(i) = *src.add(i);
        }
    }

    #[inline(always)]
    unsafe fn copy_bytes_to(&self, dst: *mut T) {
        let src: *const u8 = self.ptr.cast();
        let dst: *mut u8 = dst.cast();
        for i in 0..size_of::<T>() {
            *dst.add(i) = *src.add(i);
        }
    }

    #[inline(always)]
    pub fn new(val: T) -> Self {
        unsafe {
            let mut gb = Self::uninit();
            gb.copy_bytes_from(&val);
            forget(val);
            gb
        }
    }

    #[inline(always)]
    pub fn set(&mut self, val: T) {
        unsafe { *self.ptr = val }
    }

    #[inline(always)]
    pub fn with<T2>(self, val: T2) -> GrowBox<T2> {
        unsafe {
            drop_in_place(self.ptr);
            let mut res: GrowBox<T2> = transmute(self);

            let new_layout = min_common_layout(res.layout, Layout::new::<T2>());
            if new_layout != res.layout {
                res.set_layout(new_layout);
            }

            res.copy_bytes_from(&val);
            forget(val);
            res
        }
    }

    #[inline(always)]
    pub fn map<T2, F: Fn(T) -> T2>(self, f: F) -> GrowBox<T2> {
        unsafe {
            let mut val: T = MaybeUninit::uninit().assume_init();
            self.copy_bytes_to(&mut val);

            let new_val = f(val);
            let mut res: GrowBox<T2> = transmute(self);

            let new_layout = min_common_layout(res.layout, Layout::new::<T2>());
            if new_layout != res.layout {
                res.set_layout(new_layout);
            }

            res.copy_bytes_from(&new_val);
            forget(new_val);
            res
        }
    }
}

impl<T> Deref for GrowBox<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for GrowBox<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for GrowBox<T> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            drop_in_place(self.ptr);
            dealloc(self.ptr.cast(), self.layout);
        }
    }
}

impl<T: PartialEq> PartialEq for GrowBox<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.ptr == *other.ptr }
    }
}

impl<T: Eq> Eq for GrowBox<T> {}

impl<T: Debug> Debug for GrowBox<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        unsafe { f.debug_struct("GrowBox").field("val", &*self.ptr).finish() }
    }
}

impl<T: Clone> Clone for GrowBox<T> {
    fn clone(&self) -> Self {
        unsafe { Self::new((*self.ptr).clone()) }
    }
}
