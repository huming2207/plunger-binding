use super::eraser_error::EraserError;

pub trait BaseEraser {
    fn mass_erase(&mut self) -> Result<(), EraserError>;
    fn unlock_flash(&mut self) -> Result<(), EraserError>;
}