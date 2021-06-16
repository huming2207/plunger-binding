#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod eraser;
mod probe;
mod flasher;
mod common;
mod identifier;

use eraser::stm32l0_eraser::erase_stm32l0_async;
use flasher::generic_flasher::flash_firmware_file;
use identifier::stm32l0_identifier::identify_stm32l0_async;
use napi::{JsObject, Result};
use probe::probe_binding::get_all_probes;

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
  exports.create_named_method("eraseStm32L0", erase_stm32l0_async)?;
  exports.create_named_method("identifyStm32L0", identify_stm32l0_async)?;
  exports.create_named_method("flashFirmwareFile", flash_firmware_file)?;
  exports.create_named_method("listAllProbes", get_all_probes)?;
  Ok(())
}
