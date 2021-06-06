use std::{fs::File, u32};

use napi::{CallContext, JsBoolean, JsNumber, JsObject, JsString, Task};
use probe_rs::{DebugProbeSelector, Probe, flashing::{BinOptions, DownloadOptions, FileDownloadError, FlashLoader}};

pub struct GenericFlasherTask {
    probe_sn: Option<String>,
    firmware_type: String,
    speed_khz: Option<u32>,
    probe_vid: u16,
    probe_pid: u16,
    target_name: String,
    firmware_path: String,
    skip_erase: bool,
}

impl Task for GenericFlasherTask {
    type Output = ();
    type JsValue = JsNumber;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let mut probe = match Probe::open(DebugProbeSelector{ product_id: self.probe_pid, vendor_id: self.probe_vid, serial_number: self.probe_sn.clone() }) {
            Ok(p) => p,
            Err(err) => return Err(napi::Error{ reason: format!("Failed to open probe: {}", err), status: napi::Status::Unknown  })
        };
        
        match probe.detach() {
            Ok(_) => (),
            Err(err) => return Err(napi::Error{ reason: format!("Failed to detach probe: {}", err), status: napi::Status::Unknown  })
        };

        match probe.set_speed(self.speed_khz.unwrap_or(1800)) {
            Ok(_) => (),
            Err(err) => return Err(napi::Error{ reason: format!("Failed to set speed: {}", err), status: napi::Status::Unknown  })
        }
    
        let mut session = match probe.attach_under_reset(self.target_name.clone()) {
            Ok(s) => s,
            Err(err) => return Err(napi::Error{ reason: format!("Failed to open session: {}", err), status: napi::Status::Unknown  })
        };

        let mut file = match File::open(self.firmware_path.clone()) {
            Ok(file) => file,
            Err(err) => return Err(napi::Error{ reason: format!("Failed to open file: {}", err), status: napi::Status::InvalidArg  }),
        };

        // IMPORTANT: Change this to an actual memory map of a real chip
        let memory_map = session.target().memory_map.clone();
        let mut loader = FlashLoader::new(memory_map, probe_rs::config::TargetDescriptionSource::BuiltIn);
    
        let download_result = match self.firmware_type.as_str() {
            "bin" | "Bin" | "BIN" => {
                loader.load_bin_data(&mut file, BinOptions { base_address: None, skip: 0 })
            },
            "hex" | "IHex" | "Hex" | "ihex" | "HEX" => {
                loader.load_hex_data(&mut file)
            },
            "elf" | "Elf" | "ELF" => {
                loader.load_elf_data(&mut file)
            },
            _ => {
                Err(FileDownloadError::Object("Not a valid Bin/Hex/Elf file"))
            }
        };

        match download_result {
            Ok(_) => (),
            Err(err) => return Err(napi::Error{ reason: format!("Failed to open firmware: {}", err), status: napi::Status::Unknown  }),
        }
    
        // cb.boost_clock(&mut session)?;
    
        let mut option = DownloadOptions::new();
        if self.skip_erase {
            option.keep_unwritten_bytes = true;
            option.skip_erase = true;
        }
    
        match loader
            // TODO: hand out chip erase flag
            .commit(&mut session, option)
            .map_err(FileDownloadError::Flash) {
                Ok(_) => (),
                Err(err) => return Err(napi::Error{ reason: format!("Failed to download firmware: {}", err), status: napi::Status::Unknown  }),
            }
    
        Ok(())
    }

    fn resolve(self, env: napi::Env, _output: Self::Output) -> napi::Result<Self::JsValue> {
        // Does nothing?
        env.create_uint32(0)
    }

    fn reject(self, _env: napi::Env, err: napi::Error) -> napi::Result<Self::JsValue> {
        Err(err)
    }
}

#[js_function(7)]
pub fn flash_firmware_file(ctx: CallContext) -> napi::Result<JsObject> {
    let firmware_path = ctx.get::<JsString>(0)?.into_utf8()?.as_str()?.to_string();
    let target_name = ctx.get::<JsString>(1)?.into_utf8()?.as_str()?.to_string();
    let firmware_type = ctx.get::<JsString>(2)?.into_utf8()?.as_str()?.to_string();
    let skip_erase = match ctx.try_get::<JsBoolean>(3)? {
        napi::Either::A(erase) => erase.get_value().unwrap_or(false),
        napi::Either::B(_) => false,
    };
    let vid = ctx.get::<JsNumber>(4)?.get_int32()?;
    let pid = ctx.get::<JsNumber>(5)?.get_int32()?;
    let speed_khz = match ctx.try_get::<JsNumber>(6)? {
        napi::Either::A(sn) => Some(sn.get_uint32().unwrap_or(1800)),
        napi::Either::B(_) => None,
    };
    let serial_num = match ctx.try_get::<JsString>(7)? {
        napi::Either::A(sn) => Some(sn.into_utf8()?.as_str()?.to_string()),
        napi::Either::B(_) => None,
    };

    if vid > u16::MAX as i32 || pid > u16::MAX as i32 {
        return Err(napi::Error{ status: napi::Status::InvalidArg, reason: "Invalid VID/PID provided".to_string() });
    }

    let task = GenericFlasherTask { probe_sn: serial_num.clone(), probe_vid: vid as u16, probe_pid: pid as u16, target_name: target_name.clone(), firmware_type, speed_khz, firmware_path, skip_erase };
    ctx.env.spawn(task).map(|t| t.promise_object())
}