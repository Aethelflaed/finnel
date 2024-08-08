#[macro_use]
pub mod table_display;

use anyhow::{Context, Result};
use chrono::{
    offset::{Local, MappedLocalTime, Utc},
    DateTime, NaiveDate, TimeZone,
};
use std::cell::OnceCell;

use finnel::Conn;

pub fn naive_date_to_utc(date: NaiveDate) -> Result<DateTime<Utc>> {
    match Local.from_local_datetime(&date.and_hms_opt(12, 0, 0).unwrap()) {
        MappedLocalTime::Single(date) => Ok(date.into()),
        MappedLocalTime::Ambiguous(date, _) => Ok(date.into()),
        MappedLocalTime::None => {
            anyhow::bail!("Impossible to map local date to UTC");
        }
    }
}

pub fn confirm() -> Result<bool> {
    println!("Do you really want to do that?");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim() == "yes")
}

pub trait DeferrableResolvedUpdateArgs<'a, U, C>: Sized {
    fn new(conn: &mut Conn, args: &'a U) -> Result<Self>;
    fn get(&'a self, conn: &mut Conn) -> Result<&C>;

    fn deferred(args: &'a U) -> DeferredUpdateArgsResolution<'a, U, Self, C> {
        DeferredUpdateArgsResolution::new(args)
    }
}

pub struct DeferredUpdateArgsResolution<'a, U, R, C> {
    args: &'a U,
    resolved_args: OnceCell<R>,
    phantom: std::marker::PhantomData<C>,
}

impl<'a, U, R, C> DeferredUpdateArgsResolution<'a, U, R, C>
where
    R: DeferrableResolvedUpdateArgs<'a, U, C>,
{
    pub fn new(args: &'a U) -> Self {
        Self {
            args,
            resolved_args: Default::default(),
            phantom: Default::default(),
        }
    }

    pub fn get(&'a self, conn: &mut Conn) -> Result<&C> {
        if self.resolved_args.get().is_none()
            && self.resolved_args.set(R::new(conn, self.args)?).is_err()
        {
            anyhow::bail!("Failed to set supposedly empty OnceCell");
        }
        self.resolved_args
            .get()
            .context("Failed to get supposedly initialized OnceCell")?
            .get(conn)
    }
}
