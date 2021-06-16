use std::time::Duration;

use napi::{CallContext, JsNumber, JsObject, JsString, JsUnknown};
use probe_rs::{DebugProbeSelector, MemoryInterface, Probe};

use crate::common::{plunger_error::PlungerError, probe_info::ProbeInfo};

use super::base_identifier::{BaseIdentifier, TargetIdentity};

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
            Ok(STM32L0Identifier{ probe: probe.into(), target_name })
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

        let mut uid_lsb: Vec<u8> = vec![0u8; 32];
        let mut uid_mid: Vec<u8> = vec![0u8; 32];
        let mut uid_msb: Vec<u8> = vec![0u8; 32];

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

        let mut flash_size_kb = vec![0 as u8, 0];
        core.read_8(STM32L0_FL_SIZE, &mut flash_size_kb)?;

        let flash_size = (((flash_size_kb[0] as usize) << 8) | (flash_size_kb[1] as usize)) * 1024;
        Ok(flash_size.into())
    }
}

struct STM32L0IdentifierTask {
    probe_sn: Option<String>,
    probe_vid: u16,
    probe_pid: u16,
    target_name: String,
}

impl napi::Task for  STM32L0IdentifierTask {
    type Output = TargetIdentity;
    type JsValue = JsUnknown;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let identifier = STM32L0Identifier::new(&ProbeInfo{ serial_num: self.probe_sn.clone(), vid: self.probe_vid, pid: self.probe_pid, probe_type: None, short_id: None }, self.target_name.clone())?;
        let unique_id = Some(identifier.get_uid()?);
        let flash_size = Some(identifier.get_flash_size()?);
        Ok(TargetIdentity{ unique_id, flash_size })
    }

    fn resolve(self, env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        env.to_js_value(&output)
    }

    fn reject(self, _env: napi::Env, err: napi::Error) -> napi::Result<Self::JsValue> {
        Err(err)
    }
}

#[js_function(4)]
pub fn identify_stm32l0_async(ctx: CallContext) -> napi::Result<JsObject> {
    let target_name = ctx.get::<JsString>(0)?.into_utf8()?.as_str()?.to_string();
    let vid = ctx.get::<JsNumber>(1)?.get_int32()?;
    let pid = ctx.get::<JsNumber>(2)?.get_int32()?;
    let serial_num = match ctx.try_get::<JsString>(3)? {
        napi::Either::A(sn) => Some(sn.into_utf8()?.as_str()?.to_string()),
        napi::Either::B(_) => None,
    };

    if vid > u16::MAX as i32 || pid > u16::MAX as i32 {
        return Err(napi::Error{ status: napi::Status::InvalidArg, reason: "Invalid VID/PID provided".to_string() });
    }

    let task = STM32L0IdentifierTask { probe_sn: serial_num.clone(), probe_vid: vid as u16, probe_pid: pid as u16, target_name: target_name.clone() };
    ctx.env.spawn(task).map(|t| t.promise_object())
}
