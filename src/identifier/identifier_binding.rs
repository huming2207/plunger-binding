use std::{collections::HashMap, sync::Mutex, time::Duration};

use lazy_static::lazy_static;
use napi::{CallContext, JsNumber, JsObject, JsString};

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

async fn identify_with_timeout(
    target_name: String,
    vid: u16,
    pid: u16,
    serial_num: Option<String>,
) -> napi::Result<TargetIdentity> {
    let handle = tokio::task::spawn_blocking(move || {
        let result = match IDENTIFIER_MAP.lock() {
            Ok(ret) => ret,
            Err(err) => {
                return Err(napi::Error {
                    status: napi::Status::Unknown,
                    reason: format!("Cannot acquire identifier map lock: {:?}", err),
                })
            }
        };

        for (key, val) in result.iter() {
            if target_name.contains(key) {
                return Ok(val(target_name.clone(), vid, pid, serial_num.clone())?);
            }
        }

        Err(napi::Error {
            status: napi::Status::Unknown,
            reason: format!("Unsupported target for identify {}", target_name),
        })
    });

    if let Ok(result) = tokio::time::timeout(Duration::from_secs(3), handle).await {
        match result {
            Ok(ret) => return ret,
            Err(err) => {
                return Err(napi::Error {
                    status: napi::Status::Unknown,
                    reason: format!("Unexpected failure to join identifier thread: {}", err),
                })
            }
        }
    } else {
        return Err(napi::Error {
            status: napi::Status::GenericFailure,
            reason: "Identifier is not responding after 3 seconds".to_owned(),
        });
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
        return Err(napi::Error {
            status: napi::Status::InvalidArg,
            reason: "Invalid VID/PID provided".to_string(),
        });
    }

    ctx.env.execute_tokio_future(
        identify_with_timeout(target_name, vid as u16, pid as u16, serial_num),
        |&mut env, data| env.to_js_value(&data),
    )
}
