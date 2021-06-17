use std::{collections::HashMap, sync::Mutex};

use lazy_static::lazy_static;
use napi::{CallContext, JsNumber, JsObject, JsString, JsUnknown};

use crate::identifier::stm32l0_identifier::identify_stm32l0;

use super::base_identifier::TargetIdentity;

type IdentifierFn = fn(String, u16, u16, Option<String>) -> napi::Result<TargetIdentity>;
type IdentifierKV = HashMap<String, IdentifierFn>;

lazy_static! {
    static ref IDENTIFIER_MAP: Mutex<IdentifierKV> = {
        let mut map: IdentifierKV = HashMap::new();
        map.insert("STM32L0".to_string(), identify_stm32l0);
        Mutex::new(map)
    };
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
        let result = match IDENTIFIER_MAP.lock() {
            Ok(ret) => ret,
            Err(err) => return Err(napi::Error{ status: napi::Status::Unknown, reason: format!("Cannot acquire identifier map lock: {:?}", err) }),
        };

        for (key, val) in result.iter() {
            if self.target_name.contains(key) {
                return Ok(val(self.target_name.clone(), self.probe_vid, self.probe_pid, self.probe_sn.clone())?);
            }
        }

        Err(napi::Error{ status: napi::Status::Unknown, reason: format!("Unsupported target {}", self.target_name) })
    }

    fn resolve(self, env: napi::Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        env.to_js_value(&output)
    }

    fn reject(self, _env: napi::Env, err: napi::Error) -> napi::Result<Self::JsValue> {
        Err(err)
    }
}

#[js_function(4)]
pub fn identify_target(ctx: CallContext) -> napi::Result<JsObject> {
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