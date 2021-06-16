export enum ProbeType {
    DapLink = "DAPLink",
    StLink = "STLink",
    Ftdi = "FTDI",
    JLink = "JLink"
}

export interface TargetIdentity {
    uniqueId?: string;
    flashSize?: number;
}

export interface ProbeInfo {
    vid: number;
    pid: number;
    serialNum?: string;
    probeType: ProbeType
    shortId: number;
}

export interface Probes {
    probes: ProbeInfo[]
}

export enum FirmwareType {
    BIN = 'bin',
    HEX = 'hex',
    ELF = 'elf'
}

export const listAllProbes: () => Probes;
export const eraseStm32L0: (targetName: string, vid: number, pid: number, serialNum?: String) => Promise<void>;
export const identifyStm32L0: (targetName: string, vid: number, pid: number, serialNum?: String) => Promise<TargetIdentity>;
export const flashFirmwareFile: (path: string, targetName: string, type: FirmwareType, vid: number, pid: number, skip_erase?: boolean, speed_khz?: number, serialNum?: string) => Promise<void>;