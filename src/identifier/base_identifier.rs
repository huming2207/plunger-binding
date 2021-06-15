use crate::common::plunger_error::PlungerError;

pub trait BaseIdentifier {
    fn get_uid(self) -> Result<u128, PlungerError>;
    fn get_flash_size(self) -> Result<usize, PlungerError>;
}