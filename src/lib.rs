// Copyright 2023 Gaosg
// Licensed under the MIT license (https://opensource.org/licenses/MIT)

#![allow(non_camel_case_types)]

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A thread-safe, immutable bag for holding any value
pub struct iBag<T: Sized> {
    inner: Arc<RwLock<T>>,
}

impl<T> iBag<T> where T: Sized {
    /// Creates a new iBag instance wrapping the given value
    ///
    /// # Examples
    /// ```
    /// use ibag::iBag;
    /// let bag = iBag::new(42);
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    /// Acquires a read lock on the contained value
    ///
    /// # Safety
    /// - The returned guard must not outlive the iBag
    /// - The guard provides thread-safe read-only access
    ///
    /// # Examples
    /// ```
    /// use ibag::iBag;
    /// let bag = iBag::new(42);
    /// let guard = bag.load();
    /// assert_eq!(*guard, 42);
    /// ```
    pub fn load(&self) -> RwLockReadGuard<T> {
        self.inner.read().unwrap()
    }

    /// Acquires a write lock on the contained value
    ///
    /// # Safety
    /// - The returned guard must not outlive the iBag
    /// - The guard provides exclusive mutable access
    ///
    /// # Examples
    /// ```
    /// use ibag::iBag;
    /// let bag = iBag::new(42);
    /// let mut guard = bag.write();
    /// *guard = 100;
    /// ```
    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.inner.write().unwrap()
    }

    /// Executes a closure with mutable access to the contained value
    ///
    /// # Thread Safety
    /// - The closure executes with exclusive access
    /// - Automatically releases the lock when done
    ///
    /// # Examples
    /// ```
    /// use ibag::iBag;
    /// let bag = iBag::new(42);
    /// bag.with(|val| *val = 100);
    /// ```
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.write();
        f(&mut *guard)
    }

    /// Executes a closure with read-only access to the contained value
    ///
    /// # Thread Safety
    /// - The closure executes with shared read access
    /// - Automatically releases the lock when done
    ///
    /// # Examples
    /// ```
    /// use ibag::iBag;
    /// let bag = iBag::new(42);
    /// let result = bag.with_read(|val| *val);
    /// assert_eq!(result, 42);
    /// ```
    pub fn with_read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.load();
        f(&*guard)
    }
}

// Automatic Clone implementation
impl<T: Sized> Clone for iBag<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// Automatic Send+Sync implementation since ArcSwap<T> is already Send+Sync
unsafe impl<T> Send for iBag<T> {}
unsafe impl<T> Sync for iBag<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::{thread, vec};

    #[test]
    fn test_basic_operations() {
        let bag = iBag::new(42);
        assert_eq!(unsafe { *bag.load() }, 42);
        
        bag.with(|val| {
            *val = 100;
        });

        assert_eq!(unsafe { *bag.load() }, 100);
    }

    #[test]
    fn test_with_closure() {
        let bag = iBag::new(String::from("test"));
        let len = bag.with(|s| s.len());
        assert_eq!(len, 4);
    }

    #[test]
    fn test_clone() {
        let bag1 = iBag::new(42);
        let bag2 = bag1.clone();
        unsafe {
            let b1 = *bag1.load();
            let b2 = *bag2.load();
            assert_eq!(b1, b2);
        }
    }

    #[test]
    fn test_thread_safety() {
        let bag = Arc::new(iBag::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let bag = bag.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    let val = unsafe { *bag.load() };
                    bag.with(|v| {
                        *v = val + 1;
                    });
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }  

    #[test]
    fn test_send_sync() {
        let bag: iBag<usize> = iBag::new(42);
        let _ = Arc::new(bag);
    }

    #[test]
    fn test_drop() {
        let bag = iBag::new(42);
        drop(bag);
    }

    #[test]
    fn test_thread_safety_with_drop() {
        let bag = iBag::new(0);
        (0..4).for_each(|i| {
            let b = bag.clone();
            thread::spawn(move || {
                let r =  b.with(|v| {
                    *v = i;
                    *v
                });
                assert_eq!(r, i);
            });
        });
    }


    #[test]
    fn test_thread_safety_with_struct() {
        struct inner {
           pub a: i32,
           pub b: i32,
           pub c: String,
        }

        let iv = inner {
            a: 0,
            b: 0,
            c: String::from("test"),
        };

        let bag = iBag::new(iv);
        (0..4).for_each(|i| {
            let b = bag.clone();
            let mut handles = vec![];
            handles.push(thread::spawn(move || {
                println!("thread: {}", i);
                let r =  b.with(|v| {
                    (*v).a = i;
                    (*v).b = i+1;
                });

                let r = b.load();
                unsafe {
                    assert_eq!((*r).a, i);
                    assert_eq!((*r).b, i+1);
                }
            }));

            for handle in handles { 
                handle.join().unwrap(); 
            }
        });
    }
}