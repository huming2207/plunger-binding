use probe_rs::DebugProbeSelector;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProbeInfo {
    pub vid: u16,
    pub pid: u16,
    pub serial_num: Option<String>,
    pub probe_type: Option<ProbeType>,
    pub short_id: Option<u32>,
}

impl From<ProbeInfo> for DebugProbeSelector {
    fn from(probe: ProbeInfo) -> Self {
        DebugProbeSelector {
            vendor_id: probe.vid,
            product_id: probe.pid,
            serial_number: probe.serial_num.clone(),
        }
    }
}

impl From<&ProbeInfo> for DebugProbeSelector {
    fn from(probe: &ProbeInfo) -> Self {
        DebugProbeSelector {
            vendor_id: probe.vid,
            product_id: probe.pid,
            serial_number: probe.serial_num.clone(),
        }
    }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub enum ProbeType {
    #[serde(rename = "DAPLink")]
    DapLink,
    #[serde(rename = "STLink")]
    StLink,
    #[serde(rename = "FTDI")]
    Ftdi,
    #[serde(rename = "JLink")]
    JLink,
}
