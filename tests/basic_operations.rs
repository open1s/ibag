use ibag::iBag;

#[test]
fn test_new() {
    let bag = iBag::new(42);
    assert_eq!(*bag.load(), 42);
}

#[test]
fn test_write() {
    let bag = iBag::new(42);
    *bag.write() = 100;
    assert_eq!(*bag.load(), 100);
}

#[test]
fn test_with() {
    let bag = iBag::new(42);
    bag.with(|val| *val = 100);
    assert_eq!(*bag.load(), 100);
}

#[test]
fn test_with_read() {
    let bag = iBag::new(42);
    let result = bag.with_read(|val| *val);
    assert_eq!(result, 42);
}