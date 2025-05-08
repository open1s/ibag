//Implement Sendable Option and Result, and support conversion to/from Option and Result


use std::sync::{Arc, Mutex};


pub enum SendableOption<T> {
    Some(Arc<Mutex<T>>),
    None,
}

impl<T> SendableOption<T> {
    pub fn new(value: T) -> Self {
        SendableOption::Some(Arc::new(Mutex::new(value)))
    }

    pub fn is_some(&self) -> bool {
        matches!(self, SendableOption::Some(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, SendableOption::None)
    }

    pub fn unwrap(self) -> Arc<Mutex<T>> {
        match self {
            SendableOption::Some(arc) => arc.clone(),
            SendableOption::None => panic!("Called `SendableOption::unwrap()` on a `None` value"),
        }
    }

    pub fn unwrap_or(self, default: T) -> Arc<Mutex<T>> {
        match self {
            SendableOption::Some(arc) => arc.clone(),
            SendableOption::None => Arc::new(Mutex::new(default)),
        }
    }

    pub fn unwrap_or_else<F>(self, f: F) -> Arc<Mutex<T>>
    where
        F: FnOnce() -> T,
    {
        match self {
            SendableOption::Some(arc) => arc.clone(),
            SendableOption::None => Arc::new(Mutex::new(f())),
        }
    }

    pub fn ok<E>(self) -> Result<Arc<Mutex<T>>,E> {
        match self {
            SendableOption::Some(arc) => Ok(arc.clone()),
            SendableOption::None => panic!("Called `SendableOption::ok()` on a `None` value"),
        }
    }

    pub fn ok_or<E>(self, err: E) -> Result<Arc<Mutex<T>>,E> {
        match self {
            SendableOption::Some(arc) => Ok(arc.clone()),
            SendableOption::None => Err(err),
        }
    }

    pub fn ok_or_else<F,E>(self, f: F) -> Result<Arc<Mutex<T>>,E>
    where
        F: FnOnce() -> E,
    {
        match self {
            SendableOption::Some(arc) => Ok(arc.clone()),
            SendableOption::None => Err(f()),
        }
    }
}

unsafe impl<T: Send> Send for SendableOption<T> {}
unsafe impl<T: Sync> Sync for SendableOption<T> {}

impl <T> Clone for SendableOption<T> {
    fn clone(&self) -> Self {
        match self {
            SendableOption::Some(arc) => SendableOption::Some(Arc::clone(arc)),
            SendableOption::None => SendableOption::None,
        }
    }
}

impl <T> Into<Option<Arc<Mutex<T>>>> for SendableOption<T> {
    fn into(self) -> Option<Arc<Mutex<T>>> {
        match self {
            SendableOption::Some(arc) => Some(arc.clone()),
            SendableOption::None => None,
        }
    }
}


impl <T> From<Option<T>> for SendableOption<T> {
    fn from(option: Option<T>) -> Self {
        match option {
            Some(value) => SendableOption::Some(Arc::new(Mutex::new(value))),
            None => SendableOption::None,
        }
    }
}


pub enum SendableResult<T, E> {
    Ok(Arc<Mutex<T>>),
    Err(E),
}

impl <T, E> SendableResult<T, E> {
    pub fn new(value: T) -> Self {
        SendableResult::Ok(Arc::new(Mutex::new(value)))
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, SendableResult::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, SendableResult::Err(_))
    }

    pub fn unwrap(self) -> Arc<Mutex<T>> {
        match self {
            SendableResult::Ok(arc) => arc.clone(),
            SendableResult::Err(_) => panic!("Called `SendableResult::unwrap()` on an `Err` value"),
        }
    }
    pub fn unwrap_err(self) -> E {
        match self {
            SendableResult::Ok(_) => panic!("Called `SendableResult::unwrap_err()` on an `Ok` value"),
            SendableResult::Err(err) => err,
        }
    }
    pub fn unwrap_or(self, default: T) -> Arc<Mutex<T>> {
        match self {
            SendableResult::Ok(arc) => arc.clone(),
            SendableResult::Err(_) => Arc::new(Mutex::new(default)),
        }
    }
    pub fn unwrap_or_else<F>(self, f: F) -> Arc<Mutex<T>>
    where
        F: FnOnce() -> T,
    {
        match self {
            SendableResult::Ok(arc) => arc.clone(),
            SendableResult::Err(_) => Arc::new(Mutex::new(f())),
        }
    }

    pub fn ok(self) -> Option<Arc<Mutex<T>>> {
        match self {
            SendableResult::Ok(arc) => Some(arc.clone()),
            SendableResult::Err(_) => None,
        }
    }

    pub fn err(self) -> Option<E> {
        match self {
            SendableResult::Ok(_) => None,
            SendableResult::Err(err) => Some(err),
        }
    }
}

unsafe impl<T: Send, E: Send> Send for SendableResult<T, E> {}
unsafe impl<T: Sync, E: Sync> Sync for SendableResult<T, E> {}

impl <T, E: Clone> Clone for SendableResult<T, E> {
    fn clone(&self) -> Self {
        match self {
            SendableResult::Ok(arc) => SendableResult::Ok(Arc::clone(arc)),
            SendableResult::Err(e) => SendableResult::Err(e.clone()),
        }
    }
}

impl <T,E> From<Result<T, E>> for SendableResult<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(value) => SendableResult::Ok(Arc::new(Mutex::new(value))),
            Err(err) => SendableResult::Err(err),
        }
    }
}

impl <T,E> Into<Result<Arc<Mutex<T>>, E>> for SendableResult<T, E> {
    fn into(self) -> Result<Arc<Mutex<T>>, E> {
        match self {
            SendableResult::Ok(arc) => Ok(arc.clone()),
            SendableResult::Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sendable_option() {
        let option = SendableOption::new(1);
        match option {
            SendableOption::Some(ref arc) => {
                let value = arc.lock().unwrap();
                assert_eq!(*value, 1);
            }
            SendableOption::None => panic!("Expected Some, got None"),
        }

        let a: SendableOption<i32> = Some(1).into();
        assert!(a.is_some());
    }

    #[test]
    fn test_sendable_result() {
        let result: SendableResult<i32, i32> = SendableResult::new(1);
        assert!(result.is_ok());

        let result: SendableResult<i32, i32> = SendableResult::Err(1);
        assert!(result.is_err());

        let result: SendableResult<i32, i32> = Ok(1).into();
        assert!(result.is_ok());
    }
}