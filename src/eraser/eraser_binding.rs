use std::{collections::HashMap, sync::Mutex};

use napi::{CallContext, JsNumber, JsObject, JsString, JsUndefined, Task};

use lazy_static::lazy_static;

use crate::eraser::{generic_eraser::erase_generic, stm32l0_eraser::erase_stm32l0};

type EraserFn = fn(String, u16, u16, Option<String>) -> napi::Result<()>;
type EraserMap = HashMap<String, EraserFn>;

lazy_static! {
    static ref ERASER_MAP: Mutex<EraserMap> = {
        let mut map: EraserMap = HashMap::new();
        map.insert("STM32L0".to_string(), erase_stm32l0);
        Mutex::new(map)
    };
}

pub struct EraserTask {
    probe_sn: Option<String>,
    probe_vid: u16,
    probe_pid: u16,
    target_name: String,
}

impl Task for EraserTask {
    type Output = ();
    type JsValue = JsUndefined;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let result = match ERASER_MAP.lock() {
            Ok(ret) => ret,
            Err(err) => {
                return Err(napi::Error {
                    status: napi::Status::Unknown,
                    reason: format!("Cannot acquire identifier map lock: {:?}", err),
                })
            }
        };

        // Search for optimised algorithm first
        for (key, val) in result.iter() {
            if self.target_name.contains(key) {
                return Ok(val(
                    self.target_name.clone(),
                    self.probe_vid,
                    self.probe_pid,
                    self.probe_sn.clone(),
                )?);
            }
        }

        // If no optimised target algorithm found, then use probe-rs's generic method
        Ok(erase_generic(
            self.probe_vid,
            self.probe_pid,
            self.probe_sn.clone(),
        )?)
    }

    fn resolve(self, env: napi::Env, _output: Self::Output) -> napi::Result<Self::JsValue> {
        // Does nothing?
        env.get_undefined()
    }

    fn reject(self, _env: napi::Env, err: napi::Error) -> napi::Result<Self::JsValue> {
        Err(err)
    }
}

#[js_function(4)]
pub fn erase_target(ctx: CallContext) -> napi::Result<JsObject> {
    let target_name = ctx.get::<JsString>(0)?.into_utf8()?.as_str()?.to_string();
    let vid = ctx.get::<JsNumber>(1)?.get_int32()?;
    let pid = ctx.get::<JsNumber>(2)?.get_int32()?;
    let serial_num = match ctx.try_get::<JsString>(3)? {
        napi::Either::A(sn) => Some(sn.into_utf8()?.as_str()?.to_string()),
        napi::Either::B(_) => None,
    };

    if vid > u16::MAX as i32 || pid > u16::MAX as i32 {
        return Err(napi::Error {
            status: napi::Status::InvalidArg,
            reason: "Invalid probe VID/PID provided".to_string(),
        });
    }

    let task = EraserTask {
        probe_sn: serial_num.clone(),
        probe_vid: vid as u16,
        probe_pid: pid as u16,
        target_name: target_name.clone(),
    };
    ctx.env.spawn(task).map(|t| t.promise_object())
}
