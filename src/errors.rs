// Copyright 2023 Brian G
// Licensed under the MIT license (https://opensource.org/licenses/MIT)

use std::error;
use std::fmt;

/// Returned when borrowing fails.
#[derive(Debug)]
pub struct InvalidThreadAccess;

impl fmt::Display for InvalidThreadAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fragile value accessed from foreign thread")
    }
}

impl error::Error for InvalidThreadAccess {}


/// Returned when borrowing fails.
#[derive(Debug)]
pub struct FailTakeOwnership;

impl fmt::Display for FailTakeOwnership {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to take ownership of value")
    }
}