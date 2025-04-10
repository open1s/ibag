use std::thread;
use ibag::iCell;

#[test]
fn test_basic_usage() {
    // Create a new iCell owned by current thread
    let cell = iCell::new(42, false);
    
    // Access value from owning thread
    assert_eq!(*cell.try_get().unwrap(), 42);
    
    // Attempt access from another thread fails
    thread::spawn(move || {
        assert!(cell.try_get().is_err());
    }).join().unwrap();
}

#[test]
fn test_ownership_transfer() {
    let cell = iCell::new("test".to_string(), false);
    
    // Transfer ownership to new thread
    let handle = thread::spawn(move || {
        assert!(cell.take_ownership().is_ok());
        assert_eq!(cell.try_get().unwrap(), "test");
        cell
    });
    
    // Get cell back from thread
    let cell = handle.join().unwrap();
    
    // Original thread no longer has access
    assert!(cell.try_get().is_err());
}

#[test]
fn test_frozen_cell() {
    let cell = iCell::new(true, true); // Create frozen cell
    
    // Attempt to take ownership fails
    thread::spawn(move || {
        assert!(cell.take_ownership().is_err());
    }).join().unwrap();
}