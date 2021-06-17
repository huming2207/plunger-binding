#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod common;
mod eraser;
mod flasher;
mod identifier;
mod probe;

use eraser::eraser_binding::erase_target;
use flasher::generic_flasher::flash_firmware_file;
use identifier::identifier_binding::identify_target;
use napi::{JsObject, Result};
use probe::probe_binding::get_all_probes;

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("eraseTarget", erase_target)?;
    exports.create_named_method("identifyTarget", identify_target)?;
    exports.create_named_method("flashFirmwareFile", flash_firmware_file)?;
    exports.create_named_method("listAllProbes", get_all_probes)?;
    Ok(())
}
