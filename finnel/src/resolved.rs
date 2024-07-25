use crate::essentials::*;

pub trait Resolvable: Sized {
    fn resolve(self, conn: &mut Conn) -> Result<Self>;
    fn as_resolved<'a>(&'a self, conn: &mut Conn) -> Result<Resolved<'a, Self>>;
}

pub fn resolve<T, F, G>(conn: &mut Conn, object: T, finder: F, getter: G) -> Result<T>
where
    F: Fn(&mut Conn, i64) -> Result<T>,
    G: Fn(&T) -> Option<i64>,
{
    if let Some(id) = getter(&object) {
        let object = finder(conn, id)?;
        resolve(conn, object, finder, getter)
    } else {
        Ok(object)
    }
}

pub fn as_resolved<'a, T, F, G>(
    conn: &mut Conn,
    object: &'a T,
    finder: F,
    getter: G,
) -> Result<Resolved<'a, T>>
where
    F: Fn(&mut Conn, i64) -> Result<T>,
    G: Fn(&T) -> Option<i64>,
{
    if let Some(id) = getter(object) {
        let object = finder(conn, id)?;
        Ok(Resolved::Replacer(resolve(conn, object, finder, getter)?))
    } else {
        Ok(Resolved::Original(object))
    }
}

pub enum Resolved<'a, T> {
    Original(&'a T),
    Replacer(T),
}

impl<'a, T> Resolved<'a, T> {
    pub fn map<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        match self {
            Resolved::Original(object) => f(object),
            Resolved::Replacer(object) => f(object),
        }
    }
}

pub fn mapresolve<'a, T>(
    conn: &mut Conn,
    maybe_object: Option<&'a T>,
) -> Result<Option<Resolved<'a, T>>>
where
    T: Resolvable,
{
    maybe_object
        .map(|object| object.as_resolved(conn))
        .transpose()
}

pub fn mapmapresolve<'a, T>(
    conn: &mut Conn,
    maybe_maybe_object: Option<Option<&'a T>>,
) -> Result<Option<Option<Resolved<'a, T>>>>
where
    T: Resolvable,
{
    maybe_maybe_object
        .map(|maybe_object| {
            maybe_object
                .map(|object| object.as_resolved(conn))
                .transpose()
        })
        .transpose()
}

pub fn mapmap<T, F, R>(maybe_object: &Option<Resolved<'_, T>>, f: F) -> Option<R>
where
    F: FnOnce(&T) -> R,
{
    maybe_object
        .as_ref()
        .map(|resolved_object| resolved_object.map(f))
}

pub fn mapmapresult<T, F, R>(maybe_object: &Option<Resolved<'_, T>>, f: F) -> Result<Option<R>>
where
    F: FnOnce(&T) -> Result<R>,
{
    maybe_object
        .as_ref()
        .map(|resolved_object| resolved_object.map(f))
        .transpose()
}

pub fn mapmapmap<T, F, R>(
    maybe_maybe_object: &Option<Option<Resolved<'_, T>>>,
    f: F,
) -> Option<Option<R>>
where
    F: FnOnce(&T) -> R,
{
    maybe_maybe_object.as_ref().map(|maybe_object| {
        maybe_object
            .as_ref()
            .map(|resolved_object| resolved_object.map(f))
    })
}

pub fn mapmapmapresult<T, F, R>(
    maybe_maybe_object: &Option<Option<Resolved<'_, T>>>,
    f: F,
) -> Result<Option<Option<R>>>
where
    F: FnOnce(&T) -> Result<R>,
{
    maybe_maybe_object
        .as_ref()
        .map(|maybe_object| {
            maybe_object
                .as_ref()
                .map(|resolved_object| resolved_object.map(f))
                .transpose()
        })
        .transpose()
}
