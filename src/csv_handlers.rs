
use std::{fs::File, path::PathBuf};
use csv::{Reader, ReaderBuilder, StringRecord};
use crate::ledger::AccountSummary;

pub struct TransactionReader {
  file_reader: Reader<File>,
}
impl TransactionReader {
  pub fn from_file(file: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
    return Ok(TransactionReader {
      file_reader: ReaderBuilder::new().from_path(file)?
    })
  }
  pub fn record(&mut self) -> Result<StringRecord, Box<dyn std::error::Error>> {
    if !self.file_reader.is_done() {
      let mut r = StringRecord::new();
      if self.file_reader.read_record(&mut r).is_ok() {
        return Ok(r)
      }
      else {
        return Err(From::from("Problem reading record."))
      }
    }
    Err(From::from("No more records!"))
  }
  pub fn is_done(&self) -> bool {
    self.file_reader.is_done()
  }
}

pub fn write_as_csv_to_stdout(account_summaries: Vec<AccountSummary>) -> Result<(), Box<dyn std::error::Error>> {
  let mut wtr = csv::Writer::from_writer(std::io::stdout());
  wtr.write_record(&["client", "available", "held", "total", "locked"])?;
  for summary in &account_summaries {
    wtr.write_record(&[
      summary.client.to_string(),
      summary.available.to_string(),
      summary.held.to_string(),
      summary.total.to_string(),
      summary.locked.to_string()
    ])?;
  }
  wtr.flush()?;
  Ok(())
}

#[cfg(test)]
mod reader_tests {
  use super::*;

  const SPEC_EXAMPLE: &str = "testdata/spec_example.csv";
  const WRONG_EXT: &str = "testdata/non_csv.txt";

  #[test]
  fn from_file_valid_csv() {
    let reader = TransactionReader::from_file(SPEC_EXAMPLE.into());
    assert!(reader.is_ok());
  }
  #[test]
  fn from_file_non_exists() {
    let reader = TransactionReader::from_file("".into());
    assert!(reader.is_err());
  }
  #[test]
  fn from_file_valid_but_wrong_ext() {
    let reader = TransactionReader::from_file(WRONG_EXT.into());
    assert!(reader.is_ok());
  }
}