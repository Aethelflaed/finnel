mod account;
mod category;
pub mod database;
mod merchant;
mod transaction;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_works() {
        use oxydized_money::{Amount, Currency::EUR, Decimal};

        let cost = Amount(Decimal::from_str_exact("-1.00").unwrap(), EUR);
        let cost2 = Amount(Decimal::from_str_exact("-1").unwrap(), EUR);
        assert_eq!(cost, cost2);
        assert_eq!("-1.00", format!("{}", cost.0));
        assert_eq!("-1", format!("{}", cost2.0));
    }
}
