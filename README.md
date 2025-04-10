# iBag - Thread-safe Immutable Bag

A Rust library providing a thread-safe, immutable bag container that allows safe concurrent access to shared data.

## Features

- Thread-safe immutable container
- Read and write operations with automatic locking
- Closure-based access patterns
- Automatic Clone, Send and Sync implementations

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ibag = "0.3.0"
```

## Usage

```rust
use ibag::iBag;

// Create a new iBag
let bag = iBag::new(42);

// Read access
let value = bag.with_read(|val| *val);
assert_eq!(value, 42);

// Write access
bag.with(|val| *val = 100);

// Thread-safe operations
let bag = Arc::new(bag);
let handles: Vec<_> = (0..10).map(|_| {
    let bag = bag.clone();
    thread::spawn(move || {
        bag.with(|val| *val += 1);
    })
}).collect();

for handle in handles {
    handle.join().unwrap();
}
```

## API Documentation

See the [API documentation](https://docs.rs/ibag) for complete details.

## License

MIT