use crate::common::plunger_error::PlungerError;

pub trait BaseEraser {
    fn mass_erase(&mut self) -> Result<(), PlungerError>;
    fn unlock_flash(&mut self) -> Result<(), PlungerError>;
}
