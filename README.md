# Plunger Node.js binding

Node.js microcontroller firmware flashing/erasing library, based on @probe-rs and @napi-rs

## API

Check this `d.ts` - detailed documentation will be provided later

```typescript
export enum ProbeType {
    DapLink = "DAPLink",
    StLink = "STLink",
    Ftdi = "FTDI",
    JLink = "JLink"
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
export const flashFirmwareFile: (path: string, targetName: string, type: FirmwareType, vid: number, pid: number, skip_erase?: boolean, speed_khz?: number, serialNum?: string) => Promise<void>;
```

## License

- Dual license:
    - GPL-3.0, for personal & research uses only
    - Commerical license for commercial propose
        - Currently licensed to:
            - SmartGuide Pty Ltd (my employer)

