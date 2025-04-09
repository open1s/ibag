use ibag::iBag;
use std::sync::Arc;
use std::thread;

#[test]
fn test_concurrent_writes() {
    let bag = Arc::new(iBag::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let bag = bag.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                bag.with(|v| *v += 1);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*bag.load(), 10000);
}

#[test]
fn test_drop_behavior() {
    let bag = iBag::new(0);
    drop(bag);
}

#[test]
fn test_clone_independence() {
    let bag1 = iBag::new(42);
    let bag2 = bag1.clone();
    *bag1.write() = 100;
    assert_eq!(*bag2.load(), 100);
}