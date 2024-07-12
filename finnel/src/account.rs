use crate::{essentials::*, schema::accounts, Amount, Currency, Decimal};

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

impl NewAccount<'_> {
    pub fn new<'a>(name: &'a str) -> NewAccount<'a> {
        NewAccount {
            name,
            balance: Decimal::ZERO,
            currency: Currency::EUR,
        }
    }

    pub fn save(self, conn: &mut Conn) -> Result<Account> {
        Ok(diesel::insert_into(accounts::table)
            .values(self)
            .returning(Account::as_returning())
            .get_result(conn)?)
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
