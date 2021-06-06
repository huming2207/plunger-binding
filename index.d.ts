export enum ProbeType {
    CmsisDap = "CmsisDap",
    StLink = "StLink",
    Ftdi = "Ftdi",
    JLink = "JLink"
}

export interface ProbeInfo {
    vid: number;
    pid: number;
    serialNum?: string;
    probeType: ProbeType
}

export interface Probes {
    probes: ProbeInfo[]
}

export const listAllProbes: () => Probes;
