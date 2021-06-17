use probe_rs::{DebugProbeSelector, MemoryInterface, Probe};
use std::time::Duration;

use crate::common::{plunger_error::PlungerError, probe_info::ProbeInfo};

use super::base_identifier::{BaseIdentifier, TargetIdentity};

const STM32L0_UID_LSB: u32 = 0x1ff80050;
const STM32L0_UID_MID: u32 = 0x1ff80054;
const STM32L0_UID_MSB: u32 = 0x1ff80058;
const STM32L0_FL_SIZE: u32 = 0x1ff8007c;

pub struct STM32L0Identifier {
    probe: DebugProbeSelector,
    target_name: String,
}

impl STM32L0Identifier {
    pub fn new(probe: &ProbeInfo, target_name: String) -> Result<STM32L0Identifier, PlungerError> {
        if !target_name.contains("STM32L0") && !target_name.contains("stm32l0") {
            return Err(PlungerError::InvalidTarget(format!(
                "Target {} is not STM32L0!",
                target_name
            )));
        } else {
            Ok(STM32L0Identifier {
                probe: probe.into(),
                target_name,
            })
        }
    }
}

impl BaseIdentifier for STM32L0Identifier {
    fn get_uid(&self) -> Result<Vec<u8>, PlungerError> {
        let mut probe = Probe::open(self.probe.clone())?;

        probe.detach()?;

        let mut session = probe.attach(self.target_name.clone())?;
        let mut core = session.core(0)?;

        if !core.core_halted()? {
            core.halt(Duration::from_secs(1))?;
        }

        let mut uid_lsb: Vec<u8> = vec![0u8; 4];
        let mut uid_mid: Vec<u8> = vec![0u8; 4];
        let mut uid_msb: Vec<u8> = vec![0u8; 4];

        core.read_8(STM32L0_UID_LSB, &mut uid_lsb)?;
        core.read_8(STM32L0_UID_MID, &mut uid_mid)?;
        core.read_8(STM32L0_UID_MSB, &mut uid_msb)?;

        let mut uid: Vec<u8> = vec![];
        uid.append(&mut uid_lsb);
        uid.append(&mut uid_mid);
        uid.append(&mut uid_msb);

        return Ok(uid);
    }

    fn get_flash_size(&self) -> Result<usize, PlungerError> {
        let mut probe = Probe::open(self.probe.clone())?;

        probe.detach()?;

        let mut session = probe.attach(self.target_name.clone())?;
        let mut core = session.core(0)?;

        if !core.core_halted()? {
            core.halt(Duration::from_secs(1))?;
        }

        let mut flash_size_kb = vec![0u8; 2];
        core.read_8(STM32L0_FL_SIZE, &mut flash_size_kb)?;

        let flash_size = (((flash_size_kb[1] as usize) << 8) | (flash_size_kb[0] as usize)) * 1024;
        Ok(flash_size.into())
    }
}

pub(crate) fn identify_stm32l0(
    target_name: String,
    vid: u16,
    pid: u16,
    sn: Option<String>,
) -> napi::Result<TargetIdentity> {
    let identifier = STM32L0Identifier::new(
        &ProbeInfo {
            serial_num: sn.clone(),
            vid,
            pid,
            probe_type: None,
            short_id: None,
        },
        target_name.clone(),
    )?;
    let unique_id = Some(identifier.get_uid()?);
    let flash_size = Some(identifier.get_flash_size()?);
    Ok(TargetIdentity {
        unique_id,
        flash_size,
    })
}
