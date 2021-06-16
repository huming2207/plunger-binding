use std::usize;

use crate::common::plunger_error::PlungerError;
use serde::{Serialize, Deserialize};
use serde_with::{serde_as, hex::Hex};

pub trait BaseIdentifier {
    fn get_uid(&self) -> Result<Vec<u8>, PlungerError>;
    fn get_flash_size(&self) -> Result<usize, PlungerError>;
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetIdentity {
    #[serde_as(as = "Option<Hex>")]
    pub unique_id: Option<Vec<u8>>,
    pub flash_size: Option<usize>,
}