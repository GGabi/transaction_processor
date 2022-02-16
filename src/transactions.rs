
use std::str::FromStr;
use csv::StringRecord;

use crate::{ClientId, TxnId, Currency};

#[derive(Clone, Debug, PartialEq)]
pub enum BasicTransaction {
    Deposit    { client_id: ClientId, txn_id: TxnId, amount: Currency, disputed: bool },
    Withdrawal { client_id: ClientId, txn_id: TxnId, amount: Currency, disputed: bool },
}
impl BasicTransaction {
    pub fn new_dep(client_id: ClientId, txn_id: TxnId, amount: Currency) -> Self {
        Self::Deposit { client_id, txn_id, amount, disputed: false }
    }
    pub fn new_wit(client_id: ClientId, txn_id: TxnId, amount: Currency) -> Self {
        Self::Withdrawal { client_id, txn_id, amount, disputed: false }
    }
    pub fn client_id(&self) -> ClientId {
        match self {
            &Self::Deposit    { client_id, .. } => client_id,
            &Self::Withdrawal { client_id, .. } => client_id,
        }
    }
    pub fn txn_id(&self) -> TxnId {
        match self {
            &Self::Deposit    { client_id: _, txn_id, .. } => txn_id,
            &Self::Withdrawal { client_id: _, txn_id, .. } => txn_id,
        }
    }
    pub fn amount(&self) -> Currency {
        match &self {
            &Self::Deposit    { client_id: _, txn_id: _, amount, .. } => amount.clone(),
            &Self::Withdrawal { client_id: _, txn_id: _, amount, .. } => amount.clone(),
        }
    }
    pub fn disputed(&self) -> bool {
        match self {
            &Self::Deposit    { client_id: _, txn_id: _, amount: _, disputed } => disputed,
            &Self::Withdrawal { client_id: _, txn_id: _, amount: _, disputed } => disputed,
        }
    }
    pub fn set_disputed(&mut self, new_state: bool) {
      match self {
        Self::Deposit    { client_id: _, txn_id: _, amount: _, disputed } => *disputed = new_state,
        Self::Withdrawal { client_id: _, txn_id: _, amount: _, disputed } => *disputed = new_state,
      }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReferentialTransaction {
  Dispute    { client_id: ClientId, txn_id: TxnId },
  Resolve    { client_id: ClientId, txn_id: TxnId },
  Chargeback { client_id: ClientId, txn_id: TxnId }
}
impl ReferentialTransaction {
  pub fn new_dis(client_id: ClientId, txn_id: TxnId) -> Self {
    ReferentialTransaction::Dispute { client_id, txn_id }
  }
  pub fn new_res(client_id: ClientId, txn_id: TxnId) -> Self {
    ReferentialTransaction::Resolve { client_id, txn_id }
  }
  pub fn new_cha(client_id: ClientId, txn_id: TxnId) -> Self {
    ReferentialTransaction::Chargeback { client_id, txn_id }
  }
  pub fn client_id(&self) -> ClientId {
    match self {
      Self::Dispute    { client_id, .. } => *client_id,
      Self::Resolve    { client_id, .. } => *client_id,
      Self::Chargeback { client_id, .. } => *client_id,
    }
  }
  pub fn txn_id(&self) -> TxnId {
      match self {
        Self::Dispute    { client_id: _, txn_id, .. } => *txn_id,
        Self::Resolve    { client_id: _, txn_id, .. } => *txn_id,
        Self::Chargeback { client_id: _, txn_id, .. } => *txn_id,
      }
  }
}

#[derive(Clone, Debug)]
pub enum Transaction {
  Basic(BasicTransaction),
  Referential(ReferentialTransaction),
}
impl Transaction {
  pub fn new_dep(client_id: ClientId, txn_id: TxnId, amount: Currency) -> Self {
    Self::Basic(BasicTransaction::new_dep(client_id, txn_id, amount))
  }
  pub fn new_wit(client_id: ClientId, txn_id: TxnId, amount: Currency) -> Self {
    Self::Basic(BasicTransaction::new_wit(client_id, txn_id, amount))
  }
  pub fn new_dis(client_id: ClientId, txn_id: TxnId) -> Self {
    Self::Referential(ReferentialTransaction::new_dis(client_id, txn_id))
  }
  pub fn new_res(client_id: ClientId, txn_id: TxnId) -> Self {
    Self::Referential(ReferentialTransaction::new_res(client_id, txn_id))
  }
  pub fn new_cha(client_id: ClientId, txn_id: TxnId) -> Self {
    Self::Referential(ReferentialTransaction::new_cha(client_id, txn_id))
  }
  pub fn client_id(&self) -> ClientId {
    match &self {
      Self::Basic(txn)    => txn.client_id(),
      Self::Referential(txn) => txn.client_id()
    }
  }
  pub fn txn_id(&self) -> TxnId {
    match &self {
      Self::Basic(txn)    => txn.txn_id(),
      Self::Referential(txn) => txn.txn_id()
    }
  }
  pub fn amount(&self) -> Option<Currency> {
    if let Self::Basic(txn) = self { Some(txn.amount()) } else { None }
  }
  pub fn disputed(&self) -> Option<bool> {
    if let Self::Basic(txn) = self { Some(txn.disputed()) } else { None }
  }
  pub fn is_basic(&self) -> bool {
    if let Self::Basic(_) = self { true } else { false }
  }
  pub fn into_inner_basic(self) -> Option<BasicTransaction> {
    if let Self::Basic(txn) = self { Some(txn) } else { None }
  }
}
impl std::convert::TryFrom<StringRecord> for Transaction {
  type Error = ();
  fn try_from(string_record: StringRecord) -> Result<Self, Self::Error> {
    if string_record.len() < 3 {
      return Err(())
    }
    let client_id = if let Some(client_id) = string_record.get(1) {
        if let Ok(client_id) = client_id.trim().parse::<ClientId>() {
          client_id
        }
        else {
          return Err(())
        }
      } else {
        return Err(())
    };
    let txn_id = if let Some(txn_id) = string_record.get(2) {
        if let Ok(txn_id) = txn_id.trim().parse::<TxnId>() {
          txn_id
        }
        else {
          return Err(())
        }
      } else {
        return Err(())
    };
    let amount = if let Some(amount) = string_record.get(3) { Currency::from_str(amount) } else { Err(bigdecimal::ParseBigDecimalError::Empty) };
    // Unwrap safety: already checked that string_record has a length > 2
    match (string_record.get(0).unwrap().trim(), amount) {
      ("deposit",    Ok(amount)) => Ok(Transaction::new_dep(client_id, txn_id, amount)),
      ("withdrawal", Ok(amount)) => Ok(Transaction::new_wit(client_id, txn_id, amount)),
      ("dispute",    Err(_)) => Ok(Transaction::new_dis(client_id, txn_id)),
      ("resolve",    Err(_)) => Ok(Transaction::new_res(client_id, txn_id)),
      ("chargeback", Err(_)) => Ok(Transaction::new_cha(client_id, txn_id)),
      _ => Err(())
    }
  }
}
