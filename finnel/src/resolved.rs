pub enum Resolved<'a, T, E> {
    Original(&'a T),
    Replacer(T),
    Err(E),
}

impl<'a, T, E> Resolved<'a, T, E> {
    pub fn map<F, R>(self, f: F) -> std::result::Result<R, E>
    where
        F: FnOnce(&T) -> R,
    {
        match self {
            Resolved::Original(object) => Ok(f(object)),
            Resolved::Replacer(object) => Ok(f(&object)),
            Resolved::Err(e) => Err(e),
        }
    }
}

impl<T, E> From<Result<T, E>> for Resolved<'_, T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(o) => Resolved::Replacer(o),
            Err(e) => Resolved::Err(e),
        }
    }
}
