pub trait ResultExt<T>{
    fn map_either<U, F>(self, cb: F) -> Result<U, U> where F: FnOnce(T) -> U;
    fn either(self) -> T;
}

impl<T> ResultExt<T> for Result<T, T>{
    fn map_either<U, F>(self, cb: F) -> Result<U, U> where F: FnOnce(T) -> U {
        match self{
            Ok(a) => Ok(cb(a)),
            Err(b) => Err(cb(b)),
        }
    }

    fn either(self) -> T{
        match self {
            Ok(a) => a,
            Err(b) => b,
        }
    }
}