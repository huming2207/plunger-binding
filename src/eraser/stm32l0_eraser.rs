use std::{thread, time::Duration};

use probe_rs::{Core, DebugProbeSelector, MemoryInterface, Probe};

use super::{base_eraser::BaseEraser, eraser_error::EraserError};

use napi::{CallContext, JsNumber, JsObject, JsString, Task};

const FLASH_PECR: u32 = 0x40022004;
const FLASH_PKEYR: u32 = 0x4002200C;
const FLASH_PRGKEYR: u32 = 0x40022010;
const FLASH_OPTKEYR: u32 = 0x40022014;
const FLASH_SR: u32 = 0x40022018;
const FLASH_OPTR: u32 = 0x4002201C;
const FLASH_OPT_BASE: u32 = 0x1ff80000;

pub struct STM32L0Eraser {
    probe: DebugProbeSelector,
    target_name: String
}

impl STM32L0Eraser {
    pub fn new(target_name: String, probe: DebugProbeSelector) -> Result<STM32L0Eraser, EraserError> {
        if !target_name.contains("STM32L0") && !target_name.contains("stm32l0") {
            return Err(EraserError::InvalidTarget);
        }

        Ok(STM32L0Eraser { target_name, probe: probe.clone() })
    }

    fn wait_for_flash(core: &mut Core) -> Result<(), EraserError> {
        let mut result: u32 = 1;
        while result != 0 {
            result = core.read_word_32(FLASH_SR)? & 0b1;
        }

        Ok(())
    }

    fn set_rdp_0_to_1(&self) -> Result<(), EraserError> {
        // Prepare the probe
        let mut probe = Probe::open(self.probe.clone())?;
        
        probe.detach()?;

        let mut session = probe.attach(self.target_name.clone())?;
        let mut core = session.core(0)?;
        
        core.halt(Duration::from_secs(1))?;
        
        // Enable erasing
        core.write_word_32(FLASH_PECR, 0x200)?;
        STM32L0Eraser::wait_for_flash(&mut core)?;
        
        // Erase OPT1
        core.write_word_32(FLASH_OPT_BASE, 0)?;
        STM32L0Eraser::wait_for_flash(&mut core)?;

        // Set OBR_LAUNCH to commit (and also reboot)
        core.write_word_32(FLASH_PECR, 0x40000)?;

        // Maybe this wait is needed, but looks like without it also works...
        thread::sleep(Duration::from_micros(200));

        Ok(())
    }

    fn get_option_byte(&self) -> Result<u32, EraserError> {
        // Prepare the probe
        let mut probe = Probe::open(self.probe.clone())?;
        probe.detach()?;

        let mut session = probe.attach(self.target_name.clone())?;
        let mut core = session.core(0)?;

        // Read OPTR for RDP level
        Ok(core.read_word_32(FLASH_OPTR)?)
    }
}

impl BaseEraser for STM32L0Eraser {
    fn mass_erase(&mut self) -> Result<(), EraserError> {

        // Firstly, unlock the flash
        self.unlock_flash()?;

        let opt_val = self.get_option_byte()?;


        // RDP = 0xCC => RDP level 2, fully protected
        if opt_val & 0xff == 0xCC {
            return Err(EraserError::InvalidProtectionLevel);
        } 

        // RDP = 0xAA => RDP level 0, default
        if opt_val & 0xff == 0xAA {
            println!("Setting RDP 0 to 1");
            self.set_rdp_0_to_1()?;

            // Re-unlock the flash for the next step
            self.unlock_flash()?;
        }

        // RDP with other values (or previously been set as 1) => deal it as 1
        // Prepare the probe
        let mut probe = Probe::open(self.probe.clone())?;
        probe.detach()?;

        let mut session = probe.attach(self.target_name.clone())?;
        let mut core = session.core(0)?;


        println!("Setting RDP 1 to 0");
        let mut opt_lsb = opt_val & 0xffff;
        opt_lsb &= !0xff;
        opt_lsb |= 0xaa;
        let opt_write = (!opt_lsb) << 16 | opt_lsb;

        core.write_word_32(FLASH_OPT_BASE, opt_write)?;
        STM32L0Eraser::wait_for_flash(&mut core)?;

        // Set OBR_LAUNCH
        core.write_word_32(FLASH_PECR, 0x40000)?;

        // Maybe this wait is needed, but looks like without it also works...
        thread::sleep(Duration::from_micros(200));

        Ok(())
    }

    fn unlock_flash(&mut self) -> Result<(), EraserError> {
        let probe = Probe::open(self.probe.clone())?;

        let mut session = probe.attach_under_reset(self.target_name.clone())?;
        let mut core = session.core(0)?;

        core.halt(Duration::from_secs(1))?;

        // Unlock flash PKEY
        core.write_word_32(FLASH_PKEYR, 0x89abcdef)?;
        core.write_word_32(FLASH_PKEYR, 0x02030405)?;
        STM32L0Eraser::wait_for_flash(&mut core)?;

        // Unlock PRGKEY - programming
        core.write_word_32(FLASH_PRGKEYR, 0x8c9daebf)?;
        core.write_word_32(FLASH_PRGKEYR, 0x13141516)?;
        STM32L0Eraser::wait_for_flash(&mut core)?;


        // Unlock OPTKEY - option bytes 
        core.write_word_32(FLASH_OPTKEYR, 0xfbead9c8)?;
        core.write_word_32(FLASH_OPTKEYR, 0x24252627)?;
        STM32L0Eraser::wait_for_flash(&mut core)?;

        Ok(())
    }
}

pub struct Stm32L0EraserTask {
    probe_sn: Option<String>,
    probe_vid: u16,
    probe_pid: u16,
    target_name: String,
}

impl Task for Stm32L0EraserTask {
    type Output = ();
    type JsValue = JsNumber;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let mut eraser = match STM32L0Eraser::new(self.target_name.clone(), DebugProbeSelector{ serial_number: self.probe_sn.clone(), vendor_id: self.probe_vid, product_id: self.probe_pid }) {
            Ok(eraser) => eraser,
            Err(err) => return Err(match err {
                EraserError::InvalidTarget => napi::Error{ reason: "Invalid target".to_string(), status: napi::Status::InvalidArg },
                EraserError::InvalidProtectionLevel => napi::Error{ reason: "Invalid RDP level".to_string(), status: napi::Status::PendingException },
                EraserError::SessionError(_) => napi::Error{ reason: "Invalid session".to_string(), status: napi::Status::PendingException },
                EraserError::DebugProbeError(_) => napi::Error{ reason: "Something wrong with debug probe".to_string(), status: napi::Status::PendingException },
            }),
        };

        return match eraser.mass_erase() {
            Ok(_) => Ok(()),
            Err(err) => Err(match err {
                EraserError::InvalidTarget => napi::Error{ reason: "Invalid target".to_string(), status: napi::Status::InvalidArg },
                EraserError::InvalidProtectionLevel => napi::Error{ reason: "Invalid RDP level".to_string(), status: napi::Status::PendingException },
                EraserError::SessionError(_) => napi::Error{ reason: "Invalid session".to_string(), status: napi::Status::PendingException },
                EraserError::DebugProbeError(_) => napi::Error{ reason: "Something wrong with debug probe".to_string(), status: napi::Status::PendingException },
            }),
        };
    }

    fn resolve(self, env: napi::Env, _output: Self::Output) -> napi::Result<Self::JsValue> {
        // Does nothing?
        env.create_uint32(0)
    }

    fn reject(self, _env: napi::Env, err: napi::Error) -> napi::Result<Self::JsValue> {
        Err(err)
    }
}

#[js_function(4)]
pub fn erase_stm32l0_async(ctx: CallContext) -> napi::Result<JsObject> {
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

    let task = Stm32L0EraserTask { probe_sn: serial_num.clone(), probe_vid: vid as u16, probe_pid: pid as u16, target_name: target_name.clone() };
    ctx.env.spawn(task).map(|t| t.promise_object())
}
