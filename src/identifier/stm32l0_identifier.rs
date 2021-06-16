use std::time::Duration;

use probe_rs::{DebugProbeSelector, MemoryInterface, Probe};

use crate::common::{plunger_error::PlungerError, probe_info::ProbeInfo};

use super::base_identifier::BaseIdentifier;

const STM32L0_UID_LSB: u32 = 0x1ff80050;
const STM32L0_UID_MID: u32 = 0x1ff80054;
const STM32L0_UID_MSB: u32 = 0x1ff80058;
const STM32L0_FL_SIZE: u32 = 0x1ff8007c;

pub struct STM32L0Identifier {
    probe: DebugProbeSelector,
    target_name: String
}

impl STM32L0Identifier {
    pub fn new(probe: &ProbeInfo, target_name: String) -> Result<STM32L0Identifier, PlungerError> {
        if !target_name.contains("STM32L0") && !target_name.contains("stm32l0") {
            return Err(PlungerError::InvalidTarget);
        } else {
            Ok(STM32L0Identifier{ probe: DebugProbeSelector{ product_id: probe.pid, vendor_id: probe.vid, serial_number: probe.serial_num.clone() }, target_name })
        }
    }
}

impl BaseIdentifier for STM32L0Identifier {
    fn get_uid(self) -> Result<u128, PlungerError> {
        let mut probe = Probe::open(self.probe.clone())?;
        
        probe.detach()?;

        let mut session = probe.attach(self.target_name.clone())?;
        let mut core = session.core(0)?;
        
        if !core.core_halted()? {
            core.halt(Duration::from_secs(1))?;
        }

        let uid_lsb = core.read_word_32(STM32L0_UID_LSB)?;
        let uid_mid = core.read_word_32(STM32L0_UID_MID)?;
        let uid_msb = core.read_word_32(STM32L0_UID_MSB)?;

        let uid: u128 = (((uid_msb as u128) << 64) | ((uid_mid as u128) << 32) | (uid_lsb as u128)).into();
        Ok(uid)
    }

    fn get_flash_size(self) -> Result<usize, PlungerError> {
        let mut probe = Probe::open(self.probe.clone())?;
        
        probe.detach()?;

        let mut session = probe.attach(self.target_name.clone())?;
        let mut core = session.core(0)?;
        
        if !core.core_halted()? {
            core.halt(Duration::from_secs(1))?;
        }

        let mut flash_size_kb = vec![0 as u8, 0];
        core.read_8(STM32L0_FL_SIZE, &mut flash_size_kb)?;

        let flash_size = (((flash_size_kb[0] as usize) << 8) | (flash_size_kb[1] as usize)) * 1024;
        Ok(flash_size.into())
    }
}