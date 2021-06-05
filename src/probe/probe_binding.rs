use napi::{CallContext, JsUnknown, Result};
use probe_rs::Probe;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub enum ProbeType {
    CmsisDap,
    StLink,
    Ftdi,
    JLink
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeInfo {
    vid: u16,
    pid: u16,
    serial_num: Option<String>,
    probe_type: ProbeType
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeInfoObject {
    probes: Vec<ProbeInfo>
}

#[js_function]
pub fn get_all_probes(ctx: CallContext) -> Result<JsUnknown> {
    let probes = Probe::list_all();
    let mut new_probes: Vec<ProbeInfo> = Vec::new();

    for probe in probes {
        let probe_type = match probe.probe_type {
            probe_rs::DebugProbeType::CmsisDap => ProbeType::CmsisDap,
            probe_rs::DebugProbeType::Ftdi => ProbeType::Ftdi,
            probe_rs::DebugProbeType::StLink => ProbeType::StLink,
            probe_rs::DebugProbeType::JLink => ProbeType::JLink,
        };

        let converted_probe = ProbeInfo { vid: probe.vendor_id, pid: probe.product_id, serial_num: probe.serial_number, probe_type };

        new_probes.push(converted_probe);
    }

    let value = ProbeInfoObject { probes: new_probes };

    ctx.env.to_js_value(&value)
}