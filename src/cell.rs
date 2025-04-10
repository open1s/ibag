// Copyright 2023 Brian G
// Licensed under the MIT license (https://opensource.org/licenses/MIT)

#![allow(non_camel_case_types)]

use std::cmp;
use std::fmt;
use std::mem;
use std::sync::Mutex;
use std::thread;
use std::thread::ThreadId;
use std::sync::Arc;

use crate::errors::FailTakeOwnership;
use crate::errors::InvalidThreadAccess;
use std::mem::ManuallyDrop;

/// A guard structure that tracks the ownership state of an iCell
/// This is used internally by `iCell` to enforce thread confinement.
///
/// # Fields
/// - `freeze`: Indicates if the cell is locked (ownership taken)
/// - `thread_id`: The thread that currently owns the cell
///
/// # Safety
/// The guard is protected by a `Mutex` to ensure thread-safe access.
pub struct CellGuard {
    pub freeze: bool,
    pub thread_id: ThreadId,
}

/// A thread-confined cell that enforces single-thread access to its contents
///
/// This type ensures that its contents can only be accessed from the thread that
/// currently owns it. Ownership can be transferred between threads using
/// `take_ownership()`.
///
/// # Type Parameters
/// - `T`: The type of value being stored in the cell
///
/// # Fields
/// - `value`: The wrapped value, protected by thread ownership rules
/// - `guard`: Shared state tracking the current owning thread
///
/// # Thread Safety
/// While `iCell` implements both `Send` and `Sync`, direct access to the contained
/// value is only permitted from the owning thread.
pub struct iCell<T> {
    value: ManuallyDrop<T>,
    guard: Arc<Mutex<CellGuard>>,
}

impl<T> iCell<T> {
    /// Creates a new iCell with the given value and freeze state.
    ///
    /// The new cell will be owned by the current thread. If `freeze` is `true`,
    /// the cell will be initially locked and ownership cannot be transferred
    /// until it is unfrozen.
    ///
    /// # Arguments
    /// - `value`: The value to wrap in the cell
    /// - `freeze`: Initial lock state (false = unlocked)
    ///
    /// # Returns
    /// A new `iCell` owned by the current thread.
    ///
    /// # Examples
    /// ```
    /// use ibag::iCell;
    /// let cell = iCell::new(42, false);
    /// ```
    pub fn new(value: T,freeze: bool) -> Self {
        let guard = CellGuard {
            freeze,
            thread_id: thread::current().id(),
        };

        iCell {
            value: ManuallyDrop::new(value),
            guard: Arc::new(Mutex::new(guard)),
        }
    }

    /// Attempts to take ownership of the cell from another thread.
    ///
    /// This method must be called from the thread that wants to take ownership.
    /// If successful, the cell will be marked as owned by the current thread.
    ///
    /// # Returns
    /// - `Ok(true)` if ownership was successfully transferred
    /// - `Err(FailTakeOwnership)` if the cell is already frozen
    ///
    /// # Safety
    /// The caller must ensure this is called from the new owning thread.
    ///
    /// # Examples
    /// ```
    /// use std::thread;
    /// use ibag::iCell;
    ///
    /// let cell = iCell::new(42, false);
    /// thread::spawn(move || {
    ///     cell.take_ownership().unwrap();
    ///     // Now this thread owns the cell
    /// });
    /// ```
    pub fn take_ownership(&self) -> Result<bool, FailTakeOwnership>{
        let mut guard = self.guard.lock().unwrap();
        if guard.freeze {
            return Err(FailTakeOwnership);
        }
        guard.freeze = true;
        guard.thread_id = thread::current().id();
        Ok(true)
    }

    /// Checks if the current thread is the valid owner of the cell.
    ///
    /// This is used internally to verify thread access permissions before
    /// allowing operations on the contained value.
    ///
    /// # Returns
    /// `true` if the current thread owns the cell, `false` otherwise.
    ///
    /// # Examples
    /// ```
    /// use ibag::iCell;
    /// let cell = iCell::new(42, false);
    /// assert!(cell.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        let owner = self.guard.lock().unwrap().thread_id;
        thread::current().id() == owner
    }

    #[inline(always)]
    fn assert_thread(&self) {
        if !self.is_valid() {
            panic!("trying to access wrapped value in fragile container from incorrect thread.");
        }
    }

    /// Consumes the iCell and returns the wrapped value.
    ///
    /// This is an unsafe operation that bypasses thread safety checks.
    /// The caller must ensure this is called from the owning thread.
    ///
    /// # Safety
    /// - Must be called from the owning thread (panics otherwise)
    /// - The returned value is no longer protected by thread ownership rules
    ///
    /// # Examples
    /// ```
    /// use ibag::iCell;
    /// let cell = iCell::new(42, false);
    /// let value = unsafe { cell.into_inner() };
    /// ```
    pub fn into_inner(self) -> T {
        self.assert_thread();
        let mut this = ManuallyDrop::new(self);

        unsafe { ManuallyDrop::take(&mut this.value) }
    }

    /// Attempts to consume the iCell and return the wrapped value.
    ///
    /// This is a safe alternative to `into_inner()` that returns the
    /// original cell if called from the wrong thread.
    ///
    /// # Returns
    /// - `Ok(T)` if called from the owning thread
    /// - `Err(Self)` if called from a non-owning thread
    ///
    /// # Examples
    /// ```
    /// use ibag::iCell;
    /// use std::thread;
    ///
    /// let cell = iCell::new(42, false);
    /// let result = cell.try_into_inner();
    /// assert!(result.is_ok());
    ///
    /// let cell = iCell::new(42, false);
    /// thread::spawn(move || {
    ///     let result = cell.try_into_inner();
    ///     assert!(result.is_err());
    /// });
    /// ```
    pub fn try_into_inner(self) -> Result<T, Self> {
        if self.is_valid() {
            Ok(self.into_inner())
        } else {
            Err(self)
        }
    }

    /// Returns an immutable reference to the wrapped value
    /// # Safety
    /// - Must be called from the owning thread (panics otherwise)
    fn get(&self) -> &T {
        self.assert_thread();
        &self.value
    }

    /// Returns a mutable reference to the wrapped value
    /// # Safety
    /// - Must be called from the owning thread (panics otherwise)
    fn get_mut(&mut self) -> &mut T {
        self.assert_thread();
        &mut self.value
    }

    /// Attempts to get an immutable reference to the wrapped value
    /// - Returns Ok(&T) if called from the owning thread
    /// - Returns Err(InvalidThreadAccess) if called from a non-owning thread
    /// Unlike get(), this is a safe operation that doesn't panic
    pub fn try_get(&self) -> Result<&T, InvalidThreadAccess> {
        if self.is_valid() {
            Ok(&*self.value)
        } else {
            Err(InvalidThreadAccess)
        }
    }

    /// Attempts to get a mutable reference to the wrapped value
    /// - Returns Ok(&mut T) if called from the owning thread
    /// - Returns Err(InvalidThreadAccess) if called from a non-owning thread
    /// Unlike get_mut(), this is a safe operation that doesn't panic
    pub fn try_get_mut(&mut self) -> Result<&mut T, InvalidThreadAccess> {
        if self.is_valid() {
            Ok(self.get_mut())
        } else {
            Err(InvalidThreadAccess)
        }
    }
}

impl<T> Drop for iCell<T> {
    #[track_caller]
    fn drop(&mut self) {
        if mem::needs_drop::<T>() {
            if self.is_valid() {
                unsafe { ManuallyDrop::drop(&mut self.value) };
            } else {
                panic!("destructor of fragile object ran on wrong thread");
            }
        }
    }
}

impl<T> From<T> for iCell<T> {
    #[inline]
    fn from(t: T) -> iCell<T> {
        iCell::new(t, false)
    }
}

impl<T: Clone> Clone for iCell<T> {
    #[inline]
    fn clone(&self) -> iCell<T> {
        iCell::new(self.get().clone(), false)
    }
}

impl<T: Default> Default for iCell<T> {
    #[inline]
    fn default() -> iCell<T> {
        iCell::new(T::default(), false)
    }
}

impl<T: PartialEq> PartialEq for iCell<T> {
    #[inline]
    fn eq(&self, other: &iCell<T>) -> bool {
        *self.get() == *other.get()
    }
}

impl<T: Eq> Eq for iCell<T> {}

impl<T: PartialOrd> PartialOrd for iCell<T> {
    #[inline]
    fn partial_cmp(&self, other: &iCell<T>) -> Option<cmp::Ordering> {
        self.get().partial_cmp(other.get())
    }

    #[inline]
    fn lt(&self, other: &iCell<T>) -> bool {
        *self.get() < *other.get()
    }

    #[inline]
    fn le(&self, other: &iCell<T>) -> bool {
        *self.get() <= *other.get()
    }

    #[inline]
    fn gt(&self, other: &iCell<T>) -> bool {
        *self.get() > *other.get()
    }

    #[inline]
    fn ge(&self, other: &iCell<T>) -> bool {
        *self.get() >= *other.get()
    }
}

impl<T: Ord> Ord for iCell<T> {
    #[inline]
    fn cmp(&self, other: &iCell<T>) -> cmp::Ordering {
        self.get().cmp(other.get())
    }
}

impl<T: fmt::Display> fmt::Display for iCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self.get(), f)
    }
}

impl<T: fmt::Debug> fmt::Debug for iCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.try_get() {
            Ok(value) => f.debug_struct("Fragile").field("value", value).finish(),
            Err(..) => {
                struct InvalidPlaceholder;
                impl fmt::Debug for InvalidPlaceholder {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("<invalid thread>")
                    }
                }

                f.debug_struct("Fragile")
                    .field("value", &InvalidPlaceholder)
                    .finish()
            }
        }
    }
}

// this type is sync because access can only ever happy from the same thread
// that created it originally.  All other threads will be able to safely
// call some basic operations on the reference and they will fail.
unsafe impl<T> Sync for iCell<T> {}

// The entire point of this type is to be Send
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl<T> Send for iCell<T> {}

#[test]
fn test_basic() {
    use std::thread;
    let val = iCell::new(true, false);
    assert_eq!(val.to_string(), "true");
    assert_eq!(val.get(), &true);
    assert!(val.try_get().is_ok());
    thread::spawn(move || {
        assert!(val.try_get().is_err());
    })
    .join()
    .unwrap();
}

#[test]
fn test_mut() {
    let mut val = iCell::new(true, false);
    *val.get_mut() = false;
    assert_eq!(val.to_string(), "false");
    assert_eq!(val.get(), &false);
}

#[test]
#[should_panic]
fn test_access_other_thread() {
    use std::thread;
    let val = iCell::new(true, false);
    thread::spawn(move || {
        val.get();
    })
    .join()
    .unwrap();
}

#[test]
fn test_noop_drop_elsewhere() {
    use std::thread;
    let val = iCell::new(true, false);
    thread::spawn(move || {
        // force the move
        val.try_get().ok();
    })
    .join()
    .unwrap();
}

#[test]
fn test_panic_on_drop_elsewhere() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    let was_called = Arc::new(AtomicBool::new(false));
    struct X(Arc<AtomicBool>);
    impl Drop for X {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let val = iCell::new(X(was_called.clone()), false);
    assert!(thread::spawn(move || {
        val.try_get().ok();
    })
    .join()
    .is_err());
    assert!(!was_called.load(Ordering::SeqCst));
}

#[test]
fn test_rc_sending() {
    use std::rc::Rc;
    use std::sync::mpsc::channel;
    use std::thread;

    let val = iCell::new(Rc::new(true), false);
    let (tx, rx) = channel();

    let thread = thread::spawn(move || {
        assert!(val.try_get().is_err());
        let here = val;
        tx.send(here).unwrap();
    });

    let rv = rx.recv().unwrap();
    assert!(**rv.get());

    thread.join().unwrap();
}

#[test]
fn test_rc_sending_take_ownership() {
    use std::rc::Rc;
    use std::sync::mpsc::channel;
    use std::thread;

    let val = iCell::new(Rc::new(true), false);
    let (tx, rx) = channel();

    let sender = thread::spawn(move || {
        assert!(val.try_get().is_err());
        let here = val;
        tx.send(here).unwrap();
    });

    let recv = thread::spawn(move || {
        let rv = rx.recv().unwrap();
        let r = rv.take_ownership();
        match r {
            Ok(_) => {},
            Err(_) => panic!("failed to take ownership"),
        }
        assert!(**rv.get());
    });

    recv.join().unwrap();
    sender.join().unwrap();
}