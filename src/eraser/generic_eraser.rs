use probe_rs::{config::TargetSelector, flashing::erase_all, DebugProbeSelector, Probe};

use crate::common::plunger_error::PlungerError;

use super::base_eraser::BaseEraser;

pub struct GenericEraser {
    probe: DebugProbeSelector,
}

impl GenericEraser {
    pub fn new(probe: DebugProbeSelector) -> Result<GenericEraser, PlungerError> {
        Ok(GenericEraser {
            probe: probe.clone(),
        })
    }
}

impl BaseEraser for GenericEraser {
    fn mass_erase(&mut self) -> Result<(), PlungerError> {
        // Prepare the probe
        let mut probe = Probe::open(self.probe.clone())?;

        probe.detach()?;

        let mut session = probe.attach(TargetSelector::Auto)?;

        Ok(erase_all(&mut session)?)
    }

    fn unlock_flash(&mut self) -> Result<(), PlungerError> {
        Ok(()) // No-op for now?
    }
}

pub fn erase_generic(vid: u16, pid: u16, sn: Option<String>) -> Result<(), napi::Error> {
    let mut eraser = GenericEraser::new(DebugProbeSelector {
        serial_number: sn.clone(),
        vendor_id: vid,
        product_id: pid,
    })?;
    Ok(eraser.mass_erase()?)
}
