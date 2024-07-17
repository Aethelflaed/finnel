pub use crate::schema::accounts;
use crate::{essentials::*, Amount, Currency, Decimal};

use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Account {
    pub id: i64,
    pub name: String,
    #[diesel(deserialize_as = crate::db::Decimal)]
    pub balance: Decimal,
    #[diesel(deserialize_as = crate::db::Currency)]
    pub currency: Currency,
}

impl Account {
    pub fn balance(&self) -> Amount {
        Amount(self.balance, self.currency)
    }

    pub fn find(conn: &mut Conn, id: i64) -> Result<Self> {
        accounts::table
            .find(id)
            .select(Account::as_select())
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn find_by_name(conn: &mut Conn, name: &str) -> Result<Self> {
        accounts::table
            .filter(accounts::name.eq(name))
            .select(Account::as_select())
            .first(conn)
            .map_err(|e| e.into())
    }

    /// Delete the current account, removing associated records too
    ///
    /// This method executes multiple queries without wrapping them in a
    /// transaction
    pub fn delete(&mut self, conn: &mut Conn) -> Result<()> {
        crate::record::delete_by_account_id(conn, self.id)?;
        diesel::delete(&*self).execute(conn)?;

        Ok(())
    }
}

#[derive(Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount<'a> {
    pub name: &'a str,
    #[diesel(serialize_as = crate::db::Decimal)]
    pub balance: Decimal,
    #[diesel(serialize_as = crate::db::Currency)]
    pub currency: Currency,
}

impl<'a> NewAccount<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            balance: Decimal::ZERO,
            currency: Currency::EUR,
        }
    }
}

impl NewAccount<'_> {
    pub fn save(self, conn: &mut Conn) -> Result<Account> {
        Ok(diesel::insert_into(accounts::table)
            .values(self)
            .returning(Account::as_returning())
            .get_result(conn)?)
    }
}

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = accounts)]
pub struct ChangeAccount<'a> {
    pub name: Option<&'a str>,
}

impl ChangeAccount<'_> {
    pub fn save(self, conn: &mut Conn, account: &Account) -> Result<()> {
        diesel::update(account).set(self).execute(conn)?;
        Ok(())
    }

    pub fn apply(self, conn: &mut Conn, account: &mut Account) -> Result<()> {
        self.clone().save(conn, account)?;

        if let Some(value) = self.name {
            account.name = value.to_string();
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct QueryAccount<'a> {
    pub name: Option<&'a str>,
    pub count: Option<i64>,
}

impl QueryAccount<'_> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<Account>> {
        let mut query = accounts::table.into_boxed();

        if let Some(name) = self.name {
            query = query.filter(accounts::name.like(name));
        }
        if let Some(count) = self.count {
            query = query.limit(count);
        }

        Ok(query.select(Account::as_select()).load(conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn create_then_find_by_name() -> Result<()> {
        let conn = &mut test::db()?;

        let account = NewAccount {
            name: "Bar",
            balance: Decimal::new(314, 3),
            currency: Currency::EUR,
        }
        .save(conn)?;

        assert_eq!(account.id, Account::find_by_name(conn, &account.name)?.id);
        assert_eq!(account.name, Account::find(conn, account.id)?.name);

        Ok(())
    }
}
