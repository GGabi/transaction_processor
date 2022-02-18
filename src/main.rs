#![warn(rust_2018_idioms)]

/*!
This program takes .csv files containing lists of transactions as input and outputs details of
each clients' account to stdout.

Use the following syntax to run the program:
```bash
cargo run -- "path/to/file.csv"
```
*/

mod csv_handlers;
mod transactions;
mod ledger;

use std::{
  convert::TryInto,
  env
};
use bigdecimal::BigDecimal;
use csv_handlers::{
  TransactionReader,
  write_as_csv_to_stdout
};
use ledger::Ledger;

type ClientId = u16;
type TxnId = u32;
type Currency = BigDecimal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = env::args().collect();
  let mut reader = if let Some(file_path) = args.get(1) {
      TransactionReader::from_file(file_path.into())?
    }
    else {
      return Err(From::from("Arg empty."))
  };
  let mut l = Ledger::new();
  while !reader.is_done() {
    if let Ok(record) = reader.record() {
      if let Ok(transaction) = record.try_into() { // Unfortunately, if let chains are experimental
        l.add_transaction(transaction);
      }
    }
  }
  write_as_csv_to_stdout(l.calculate_all_account_summaries())
}

#[cfg(test)]
fn new_currency(input: u32) -> Currency {
  use num::BigInt;
  // Input is scaled down by 10,000 e.g. 151234 -> 15.1234
  Currency::new(BigInt::new(num::bigint::Sign::Plus, vec![input]), 4)
}

#[cfg(test)]
mod ledger_tests {
  use super::*;
  use crate::{
    transactions::{BasicTransaction, ReferentialTransaction, Transaction},
    ledger::AccountSummary
  };
  #[test]
  fn deposit_summary_0() -> Result<(), ()> {
    let mut l = Ledger::new();
    let t = BasicTransaction::new_dep(0, 0, new_currency(10000));
    l.add_simple_transaction(t);
    let actual = l.calculate_client_account_summary(0);
    let expected = AccountSummary {
      client: 0,
      available: new_currency(10000),
      held: new_currency(0),
      total: new_currency(10000),
      locked: false,
    };
    assert_eq!(actual, Some(expected));
    Ok(())
  }
  #[test]
  fn deposit_summary_1() -> Result<(), ()> {
      let mut l = Ledger::new();
      let mut t = BasicTransaction::new_dep(0, 0, new_currency(100000));
      l.add_simple_transaction(t);
      t = BasicTransaction::new_dep(0, 1, new_currency(52500));
      l.add_simple_transaction(t);
      let actual = l.calculate_client_account_summary(0);
      let expected = AccountSummary {
          client: 0,
          available: new_currency(152500),
          held: new_currency(0),
          total: new_currency(152500),
          locked: false,
      };
      assert_eq!(actual, Some(expected));
      Ok(())
  }
  #[test]
  fn deposit_withdrawal_summary_0() -> Result<(), ()> {
      let mut l = Ledger::new();
      let mut t = BasicTransaction::new_dep(0, 0, new_currency(100000));
      l.add_simple_transaction(t);
      t = BasicTransaction::new_dep(0, 1, new_currency(52500));
      l.add_simple_transaction(t);
      t = BasicTransaction::new_wit(0, 2, new_currency(37500));
      l.add_simple_transaction(t);
      let actual = l.calculate_client_account_summary(0);
      let expected = AccountSummary {
          client: 0,
          available: new_currency(115000),
          held: new_currency(0),
          total: new_currency(115000),
          locked: false,
      };
      assert_eq!(actual, Some(expected));
      Ok(())
  }
  #[test]
  fn deposit_0() {
    let mut l = Ledger::new();
    let t = BasicTransaction::new_dep(0, 0, new_currency(100000));
    l.add_simple_transaction(t);
    assert!(l.clients.contains_key(&0));
    assert!(l.clients.get(&0).unwrap().contains(&0));
    assert!(l.txns.contains_key(&0));
    assert_eq!(BasicTransaction::new_dep(0, 0, new_currency(100000)), *l.txns.get(&0).unwrap());
  }
  #[test]
  fn multiple_summaries_0() -> Result<(), ()> {
    let mut l = Ledger::new();
    let mut t = BasicTransaction::new_dep(0, 0, new_currency(100000));
    l.add_simple_transaction(t);
    t = BasicTransaction::new_dep(1, 1, new_currency(999900));
    l.add_simple_transaction(t);
    let actual = l.calculate_all_account_summaries();
    let expected_0 = AccountSummary {
      client: 0,
      available: new_currency(100000),
      held: new_currency(0),
      total: new_currency(100000),
      locked: false,
    };
    let expected_1 = AccountSummary {
      client: 1,
      available: new_currency(999900),
      held: new_currency(0),
      total: new_currency(999900),
      locked: false,
    };
    assert!(actual.contains(&expected_0));
    assert!(actual.contains(&expected_1));
    Ok(())
  }
  #[test]
  fn deposit_dispute_0() -> Result<(), ()> {
      let mut l = Ledger::new();
      let mut t = Transaction::Basic(BasicTransaction::new_dep(0, 0, new_currency(100000)));
      l.add_transaction(t);
      t = Transaction::Basic(BasicTransaction::new_dep(0, 1, new_currency(52500)));
      l.add_transaction(t);
      t = Transaction::Referential(ReferentialTransaction::Dispute{
          client_id: 0,
          txn_id: 1,
      });
      l.add_transaction(t);
      let actual = l.calculate_client_account_summary(0);
      let expected = AccountSummary {
          client: 0,
          available: new_currency(100000),
          held: new_currency(52500),
          total: new_currency(152500),
          locked: false,
      };
      assert_eq!(actual, Some(expected));
      Ok(())
  }
  #[test]
  fn deposit_dispute_resolve_0() -> Result<(), ()> {
      let mut l = Ledger::new();
      let mut t = Transaction::Basic(BasicTransaction::new_dep(0, 0, new_currency(100000)));
      l.add_transaction(t);
      t = Transaction::Basic(BasicTransaction::new_dep(0, 1, new_currency(52500)));
      l.add_transaction(t);
      t = Transaction::Referential(ReferentialTransaction::Dispute{
          client_id: 0,
          txn_id: 1,
      });
      l.add_transaction(t);
      t = Transaction::Referential(ReferentialTransaction::Resolve{
          client_id: 0,
          txn_id: 1,
      });
      l.add_transaction(t);
      let actual = l.calculate_client_account_summary(0);
      let expected = AccountSummary {
          client: 0,
          available: new_currency(152500),
          held: new_currency(0),
          total: new_currency(152500),
          locked: false,
      };
      assert_eq!(actual, Some(expected));
      Ok(())
  }
  #[test]
  fn deposit_dispute_chargeback_0() -> Result<(), ()> {
    let mut l = Ledger::new();
    let mut t = Transaction::Basic(BasicTransaction::new_dep(0, 0, new_currency(100000)));
    l.add_transaction(t);
    t = Transaction::Basic(BasicTransaction::new_dep(0, 1, new_currency(52500)));
    l.add_transaction(t);
    t = Transaction::Referential(ReferentialTransaction::Dispute{
      client_id: 0,
      txn_id: 1,
    });
    l.add_transaction(t);
    t = Transaction::Referential(ReferentialTransaction::Chargeback{
      client_id: 0,
      txn_id: 1,
    });
    l.add_transaction(t);
    let actual = l.calculate_client_account_summary(0);
    let expected = AccountSummary {
      client: 0,
      available: new_currency(100000),
      held: new_currency(0),
      total: new_currency(100000),
      locked: true,
    };
    assert_eq!(actual, Some(expected));
    Ok(())
  }
  #[test]
  fn withdraw_dispute_0() -> Result<(), ()> {
      let mut l = Ledger::new();
      let mut t = Transaction::Basic(BasicTransaction::new_dep(0, 0, new_currency(100000)));
      l.add_transaction(t);
      t = Transaction::Basic(BasicTransaction::new_wit(0, 1, new_currency(52500)));
      l.add_transaction(t);
      t = Transaction::Referential(ReferentialTransaction::Dispute{
          client_id: 0,
          txn_id: 1,
      });
      l.add_transaction(t);
      let actual = l.calculate_client_account_summary(0);
      let expected = AccountSummary {
          client: 0,
          available: new_currency(47500),
          held: new_currency(52500),
          total: new_currency(100000),
          locked: false,
      };
      assert_eq!(actual, Some(expected));
      Ok(())
  }
  #[test]
  fn withdraw_dispute_resolve_0() -> Result<(), ()> {
      let mut l = Ledger::new();
      let mut t = Transaction::Basic(BasicTransaction::new_dep(0, 0, new_currency(100000)));
      l.add_transaction(t);
      t = Transaction::Basic(BasicTransaction::new_wit(0, 1, new_currency(52500)));
      l.add_transaction(t);
      t = Transaction::Referential(ReferentialTransaction::Dispute{
          client_id: 0,
          txn_id: 1,
      });
      l.add_transaction(t);
      t = Transaction::Referential(ReferentialTransaction::Resolve{
          client_id: 0,
          txn_id: 1,
      });
      l.add_transaction(t);
      let actual = l.calculate_client_account_summary(0);
      let expected = AccountSummary {
          client: 0,
          available: new_currency(47500),
          held: new_currency(0),
          total: new_currency(47500),
          locked: false,
      };
      assert_eq!(actual, Some(expected));
      Ok(())
  }
  #[test]
  fn withdraw_dispute_chargeback_0() -> Result<(), ()> {
      let mut l = Ledger::new();
      let mut t = Transaction::Basic(BasicTransaction::new_dep(0, 0, new_currency(100000)));
      l.add_transaction(t);
      t = Transaction::Basic(BasicTransaction::new_wit(0, 1, new_currency(52500)));
      l.add_transaction(t);
      t = Transaction::Referential(ReferentialTransaction::Dispute{
          client_id: 0,
          txn_id: 1,
      });
      l.add_transaction(t);
      t = Transaction::Referential(ReferentialTransaction::Chargeback{
          client_id: 0,
          txn_id: 1,
      });
      l.add_transaction(t);
      let actual = l.calculate_client_account_summary(0);
      let expected = AccountSummary {
          client: 0,
          available: new_currency(100000),
          held: new_currency(0),
          total: new_currency(100000),
          locked: true,
      };
      assert_eq!(actual, Some(expected));
      Ok(())
  }
}

#[cfg(test)]
mod account_summary_tests {
  use crate::ledger::AccountSummary;
  #[test]
  fn new_0() {
      let actual = AccountSummary::new();
      let expected = AccountSummary {
        client: 0,
        available: 0.into(),
        held: 0.into(),
        total: 0.into(),
        locked: false,
      };
      assert_eq!(actual, expected);
  }
}

#[cfg(test)]
mod end2end {
  use super::*;
  use bigdecimal::FromPrimitive;
  #[test]
  fn many_clients() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = TransactionReader::from_file("testdata/many_clients.csv".into())?;
    let mut l = Ledger::new();
    while !reader.is_done() {
      if let Ok(record) = reader.record() {
        if let Ok(transaction) = record.try_into() {
          l.add_transaction(transaction);
        }
      }
    }
    assert_eq!(999, l.txns.len());
    assert_eq!(999, l.clients.len());
    for (_, txn_ids) in &l.clients {
      assert_eq!(1, txn_ids.len());
    }
    assert!(l.locked_clients.is_empty());
    Ok(())
  }
  #[test]
  fn disputes() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = TransactionReader::from_file("testdata/disputes.csv".into())?;
    let mut l = Ledger::new();
    while !reader.is_done() {
      if let Ok(record) = reader.record() {
        if let Ok(transaction) = record.try_into() {
          l.add_transaction(transaction);
        }
      }
    }
    assert_eq!(27, l.txns.len());
    assert_eq!(9, l.clients.len());
    for (_, txn_ids) in &l.clients {
      assert_eq!(3, txn_ids.len());
    }
    assert!(l.locked_clients.is_empty());
    for summary in l.calculate_all_account_summaries() {
      // I'd love to assert the client_ids are correct, but can't guarantee ordering
      assert_eq!(Currency::from_f64(5.5555).unwrap(), summary.available);
      assert_eq!(Currency::from_f64(10.0).unwrap(), summary.held);
      assert_eq!(Currency::from_f64(15.5555).unwrap(), summary.total);
      assert!(!summary.locked);
    }
    Ok(())
  }
  #[test]
  fn chargeback() -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = TransactionReader::from_file("testdata/chargeback.csv".into())?;
    let mut l = Ledger::new();
    while !reader.is_done() {
      if let Ok(record) = reader.record() {
        if let Ok(transaction) = record.try_into() {
          l.add_transaction(transaction);
        }
      }
    }
    assert_eq!(3, l.txns.len());
    assert_eq!(1, l.clients.len());
    for (_, txn_ids) in &l.clients {
      assert_eq!(3, txn_ids.len());
    }
    assert_eq!(1, l.locked_clients.len());
    for summary in l.calculate_all_account_summaries() {
      // If the chargeback didn't lock the account and prevent the final
      // deposit, available would have been 7
      assert_eq!(Currency::from_f64(6.0000).unwrap(), summary.available);
      assert_eq!(Currency::from_f64(0.0).unwrap(), summary.held);
      assert_eq!(Currency::from_f64(6.0000).unwrap(), summary.total);
      assert!(summary.locked);
    }
    Ok(())
  }
}
