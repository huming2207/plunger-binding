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
