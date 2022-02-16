
use std::collections::{HashMap, HashSet, BTreeSet};

use bigdecimal::BigDecimal;

use crate::{
  ClientId,
  TxnId,
  Currency,
  transactions::{
  BasicTransaction,
  ReferentialTransaction,
  Transaction
}};

#[derive(Clone, Debug)]
pub struct Ledger {
  pub txns: HashMap<TxnId, BasicTransaction>,
  pub clients: HashMap<ClientId, BTreeSet<TxnId>>, // BTreeSet to preserve ordering of transactions (IMPORTANT!)
  pub locked_clients: HashSet<ClientId>,
}
impl Ledger {
  pub fn new() -> Self {
      Ledger {
          txns: HashMap::new(),
          clients: HashMap::new(),
          locked_clients: HashSet::new(),
      }
  }
  pub fn add_simple_transaction(&mut self, txn: BasicTransaction) {
    if !self.locked_clients.contains(&txn.client_id()) {
      if !self.clients.contains_key(&txn.client_id()) {
        self.clients.insert(txn.client_id(), BTreeSet::new());
      }
      if let Some(transaction_ids) = self.clients.get_mut(&txn.client_id()) {
        transaction_ids.insert(txn.txn_id());
        self.txns.insert(txn.txn_id(), txn);
      }
    }
  }
  pub fn add_transaction(&mut self, txn: Transaction) {
    match txn {
      Transaction::Basic(inner_txn) => return self.add_simple_transaction(inner_txn),
      Transaction::Referential(ReferentialTransaction::Dispute {client_id: _, txn_id}) =>
      if let Some(txn) = self.txns.get_mut(&txn_id) {
        txn.set_disputed(true);
      },
      Transaction::Referential(ReferentialTransaction::Resolve {client_id: _, txn_id}) =>
      if let Some(txn) = self.txns.get_mut(&txn_id) {
        txn.set_disputed(false);
      },
      Transaction::Referential(ReferentialTransaction::Chargeback{client_id, txn_id})
      if self.txns.contains_key(&txn_id)
      // Unwrap safety: Due to short-circuiting, is self.txns does not contain txn_id then self.txns.get(&txn_id).unwrap() will never be evaluated
      && self.txns.get(&txn_id).unwrap().disputed()
      && self.clients.contains_key(&client_id) => {
        self.txns.remove(&txn_id);
        // Unwrap safety: Already checked self.clients contains client_id 
        self.clients.get_mut(&client_id).unwrap().remove(&txn_id);
        self.locked_clients.insert(txn.client_id());
      },
      _ => {},
    }
  }
  pub fn calculate_all_account_summaries(&self) -> Vec<AccountSummary> {
      let mut summaries = Vec::new();
      for (&client_id, _) in &self.clients {
        if let Some(summary) = self.calculate_client_account_summary(client_id) {
          summaries.push(summary);
        }
      }
      summaries
  }
  pub fn calculate_client_account_summary(&self, client_id: ClientId) -> Option<AccountSummary> {
    // Grab transaction ids for client account
    if let Some(txn_ids) = self.clients.get(&client_id) {
      let mut acc = AccountSummary::new();
      acc.client = client_id;
      // For every transaction id, get the transaction and add if deposit else minus if withdrawal
      for txn_id in txn_ids {
        match self.txns.get(txn_id) {
          Some(BasicTransaction::Deposit{client_id: _, txn_id: _, amount, disputed: false}) => acc.available += amount.clone(),
          Some(BasicTransaction::Withdrawal{client_id: _, txn_id: _, amount, disputed: false}) if *amount <= acc.available => acc.available -= amount.clone(),
          Some(BasicTransaction::Deposit{client_id: _, txn_id: _, amount, disputed: true}) => acc.held += amount.clone(),
          Some(BasicTransaction::Withdrawal{client_id: _, txn_id: _, amount, disputed: true}) if *amount <= acc.available => {
            // Funds are still removed from available funds (transaction pending)
            // but funds placed in held until dispute resolved
            acc.available -= amount.clone();
            acc.held += amount.clone();
          },
          _ => {/* Do nothing when a withdrawal would have put account in negative balance */},
        }
      }
      acc.total = acc.available.clone() + acc.held.clone();
      acc.locked = self.locked_clients.contains(&client_id);
      Some(acc)
    }
    else {
      None
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AccountSummary {
  pub client: ClientId,
  pub available: Currency,
  pub held: Currency,
  pub total: Currency,
  pub locked: bool,
}
impl AccountSummary {
  pub fn new() -> Self {
    AccountSummary {
      client: 0,
      available: BigDecimal::new(num::zero(), 4),
      held: BigDecimal::new(num::zero(), 4),
      total: BigDecimal::new(num::zero(), 4),
      locked: false
    }
  }
}