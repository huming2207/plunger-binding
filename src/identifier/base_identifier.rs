use crate::common::plunger_error::PlungerError;

pub trait BaseIdentifier {
    fn get_uid(vid: u16, pid: u16, serial_num: u16) -> Result<u128, PlungerError>;
    fn get_flash_size() -> Result<usize, PlungerError>;
}