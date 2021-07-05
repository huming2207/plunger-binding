use crc::{Crc, CRC_32_CKSUM};
use napi::{CallContext, JsUnknown, Result};
use probe_rs::Probe;
use serde::{Deserialize, Serialize};

use crate::common::probe_info::{ProbeInfo, ProbeType};

pub const CRC: Crc<u32> = Crc::<u32>::new(&CRC_32_CKSUM);

#[derive(Serialize, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeInfoObject {
    probes: Vec<ProbeInfo>,
}

#[js_function]
pub fn get_all_probes(ctx: CallContext) -> Result<JsUnknown> {
    let probes = Probe::list_all();
    let mut new_probes: Vec<ProbeInfo> = Vec::new();

    for probe in probes {
        let probe_type = match probe.probe_type {
            probe_rs::DebugProbeType::CmsisDap => ProbeType::DapLink,
            probe_rs::DebugProbeType::Ftdi => ProbeType::Ftdi,
            probe_rs::DebugProbeType::StLink => ProbeType::StLink,
            probe_rs::DebugProbeType::JLink => ProbeType::JLink,
        };

        // An naive workaround for DAPLink initialising issue
        if matches!(probe_type, ProbeType::DapLink) && probe.serial_number.is_some() {
            if !is_daplink_ready(probe.serial_number.clone().unwrap()) {
                continue;
            }
        }

        let short_id = match &probe.serial_number {
            Some(sn) => Some(CRC.checksum(sn.as_bytes())),
            None => None,
        };

        let converted_probe = ProbeInfo {
            vid: probe.vendor_id,
            pid: probe.product_id,
            serial_num: probe.serial_number,
            probe_type: Some(probe_type),
            short_id,
        };

        new_probes.push(converted_probe);
    }

    let value = ProbeInfoObject { probes: new_probes };

    ctx.env.to_js_value(&value)
}

// Probe-rs tend to return all DAPLink probes even if it is not really ready
// Therefore it may cause some random USB stack lockup issue
// Here's a temporary fix for now...
fn is_daplink_ready(serial_num: String) -> bool {
    let mut enumerator = udev::Enumerator::new().unwrap();

    enumerator.match_property("ID_FS_LABEL", "DAPLINK").unwrap();

    for device in enumerator.scan_devices().unwrap() {
        println!("found device: {:?}", device.sysname());
        for property in device.properties() {
            if property.name().eq("ID_SERIAL_SHORT") && property.value().eq(serial_num.as_str()) {
                return true;
            }
        }
    }

    return false;
}
