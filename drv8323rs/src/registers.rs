//! Registers of the DRV8323RS, as specified in the datasheet.

use device_register::{RORegister, RWRegister};
use modular_bitfield_msb::prelude::*;

use crate::SerializableRegister;

/// The status registers are used to reporting warning and fault conditions.
#[derive(RORegister, Clone, Copy)]
#[register(addr = 0x00, ty = "u8")]
#[bitfield(bytes = 2)]
pub struct Status1 {
    #[skip]
    __: B5,
    /// Logic OR of FAULT status registers. Mirrors nFAULT pin.
    pub fault: bool,
    /// Indicates VDS monitor over-current fault condition
    pub vds_ocp: bool,
    /// Indicates gate drive fault condition
    pub gdf: bool,
    /// Indicates under-voltage lockout fault condition
    pub uvlo: bool,
    /// Indicates over-temperature shutdown
    pub otsd: bool,
    /// Indicates VDS over-current fault on the A high-side MOSFET
    pub vds_ha: bool,
    /// Indicates VDS over-current fault on the A low-side MOSFET
    pub vds_la: bool,
    /// Indicates VDS over-current fault on the B high-side MOSFET
    pub vds_hb: bool,
    /// Indicates VDS over-current fault on the B high-side MOSFET
    pub vds_lb: bool,
    /// Indicates VDS over-current fault on the C high-side MOSFET
    pub vds_hc: bool,
    /// Indicates VDS over-current fault on the C high-side MOSFET
    pub vds_lc: bool,
}

impl SerializableRegister<2> for Status1 {
    fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bytes(bytes)
    }

    fn to_bytes(self) -> [u8; 2] {
        Self::into_bytes(self)
    }
}

/// The status registers are used to reporting warning and fault conditions.
#[derive(RORegister, Clone, Copy)]
#[register(addr = 0x01, ty = "u8")]
#[bitfield(bytes = 2)]
pub struct Status2 {
    #[skip]
    __: B5,
    /// Indicates over-current on phase A sense amplifier
    pub sa_oc: bool,
    /// Indicates over-current on phase B sense amplifier
    pub sb_oc: bool,
    /// Indicates over-current on phase C sense amplifier
    pub sc_oc: bool,
    /// Indicates over-temperature warning
    pub otw: bool,
    /// Indicates charge pump under-voltage fault condition
    pub cp_uv: bool,
    /// Indicates gate drive fault on the A high-side MOSFET
    pub vgs_ha: bool,
    /// Indicates gate drive fault on the A low-side MOSFET
    pub vgs_la: bool,
    /// Indicates gate drive fault on the B high-side MOSFET
    pub vgs_hb: bool,
    /// Indicates gate drive fault on the B low-side MOSFET
    pub vgs_lb: bool,
    /// Indicates gate drive fault on the C high-side MOSFET
    pub vgs_hc: bool,
    /// Indicates gate drive fault on the C low-side MOSFET
    pub vgs_lc: bool,
}

impl SerializableRegister<2> for Status2 {
    fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bytes(bytes)
    }

    fn to_bytes(self) -> [u8; 2] {
        Self::into_bytes(self)
    }
}

/// Available PWM modes on the DRV8323RS. Part of the [DriveControl] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum PwmMode {
    /// 6x PWM Mode
    _6x = 0b00,
    /// 3x PWM Mode
    _3x = 0b01,
    /// 1x PWM Mode
    _1x = 0b10,
    /// Independent PWM Mode
    Independent = 0b11,
}

/// The control register is used to configure the DRV8323RS.
#[derive(RWRegister, Clone, Copy)]
#[register(addr = 0x02, ty = "u8")]
#[bitfield(bytes = 2)]
pub struct DriveControl {
    #[skip]
    __: B6,
    /// Charge pump disable
    pub dis_cpuv: bool,
    /// Gate drive disable
    pub dis_gdf: bool,
    /// Over-temperature report enable
    pub otw_rep: bool,
    /// PWM mode
    pub pwm_mode: PwmMode,
    /// 1x PWM mode asynchronous rectification (diode freewheeling) enable
    pub single_pwm_com: bool,
    /// In 1x PWM mode, this bit is ORed with the INHC (DIR) input
    pub single_pwm_dir: bool,
    /// Coast enable. Puts all MOSFETs in the Hi-Z state
    pub coast: bool,
    /// Break enable. Turns on all low-side MOSFETs.
    pub brake: bool,
    /// Clear latched fault bits. This is automatically reset after being written.
    pub clr_flt: bool,
}

impl SerializableRegister<2> for DriveControl {
    fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bytes(bytes)
    }

    fn to_bytes(self) -> [u8; 2] {
        Self::into_bytes(self)
    }
}

/// Register lock enable. Part of the [GateHs] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 3]
pub enum Lock {
    /// Lock the registers
    Lock = 0b110,
    /// Unlock the registers
    Unlock = 0b011,
}

/// Gate drive source current (Max: 1A). Part of the [GateHs] and [GateLs] registers.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 4]
pub enum SourceCurrent {
    /// 10 mA
    _10 = 0b0000,
    /// 30 mA
    _30 = 0b0001,
    /// 60 mA
    _60 = 0b0010,
    /// 80 mA
    _80 = 0b0011,
    /// 120 mA
    _120 = 0b0100,
    /// 140 mA
    _140 = 0b0101,
    /// 170 mA
    _170 = 0b0110,
    /// 190 mA
    _190 = 0b0111,
    /// 260 mA
    _260 = 0b1000,
    /// 330 mA
    _330 = 0b1001,
    /// 370 mA
    _370 = 0b1010,
    /// 440 mA
    _440 = 0b1011,
    /// 570 mA
    _570 = 0b1100,
    /// 680 mA
    _680 = 0b1101,
    /// 820 mA
    _820 = 0b1110,
    /// 1000 mA
    _1000 = 0b1111,
}

/// Gate drive sink current (Max: 2A). Part of the [GateHs] and [GateLs] registers.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 4]
pub enum SinkCurrent {
    /// 20 mA
    _20 = 0b0000,
    /// 60 mA
    _60 = 0b0001,
    /// 120 mA
    _120 = 0b0010,
    /// 160 mA
    _160 = 0b0011,
    /// 240 mA
    _240 = 0b0100,
    /// 280 mA
    _280 = 0b0101,
    /// 340 mA
    _340 = 0b0110,
    /// 380 mA
    _380 = 0b0111,
    /// 520 mA
    _520 = 0b1000,
    /// 660 mA
    _660 = 0b1001,
    /// 740 mA
    _740 = 0b1010,
    /// 880 mA
    _880 = 0b1011,
    /// 1140 mA
    _1140 = 0b1100,
    /// 1360 mA
    _1360 = 0b1101,
    /// 1640 mA
    _1640 = 0b1110,
    /// 2000 mA
    _2000 = 0b1111,
}

/// The gate drive high-side register.
#[derive(RWRegister, Clone, Copy)]
#[register(addr = 0x03, ty = "u8")]
#[bitfield(bytes = 2)]
pub struct GateHs {
    #[skip]
    __: B5,
    /// Whether to lock all registers (except this field and coast, brake, and clear bits of [DriveControl]).
    pub lock: Lock,
    /// High side source current
    pub idrivep_hs: SourceCurrent,
    /// High side sink current
    pub idriven_hs: SinkCurrent,
}

impl SerializableRegister<2> for GateHs {
    fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bytes(bytes)
    }

    fn to_bytes(self) -> [u8; 2] {
        Self::into_bytes(self)
    }
}

/// Peak drive time. Part of the [GateLs] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum TDrive {
    /// 500 ns peak gate-current drive time
    _500 = 0b00,
    /// 1000 ns peak gate-current drive time
    _1000 = 0b01,
    /// 2000 ns peak gate-current drive time
    _2000 = 0b10,
    /// 4000 ns peak gate-current drive time
    _4000 = 0b11,
}

/// The gate drive low-side register.
#[derive(RWRegister, Clone, Copy)]
#[register(addr = 0x04, ty = "u8")]
#[bitfield(bytes = 2)]
pub struct GateLs {
    #[skip]
    __: B5,
    /// Cycle-by-cycle operation. In retry OCP_MODE, for both VDS_COP and SEN_OCP,
    /// the fault is automatically cleared when a PWM input is given.
    pub cbc: bool,
    /// Peak drive time
    pub t_drive: TDrive,
    /// Low side source current
    pub idrivep_ls: SourceCurrent,
    /// Low side sink current
    pub idriven_ls: SinkCurrent,
}

impl SerializableRegister<2> for GateLs {
    fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bytes(bytes)
    }

    fn to_bytes(self) -> [u8; 2] {
        Self::into_bytes(self)
    }
}

/// Retry time. Part of the [OcpControl] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 1]
pub enum TRetry {
    /// Retry after 4 ms
    _4ms = 0b0,
    /// Retry after 50 µs
    _50us = 0b1,
}

/// Dead time. Part of the [OcpControl] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum DeadTime {
    /// 50 ns dead time
    _50 = 0b00,
    /// 100 ns dead time
    _100 = 0b01,
    /// 200 ns dead time
    _200 = 0b10,
    /// 400 ns dead time
    _400 = 0b11,
}

/// Over-current protection mode. Part of the [OcpControl] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum OcpMode {
    /// Over-current causes a latched fault
    Latched = 0b00,
    /// Over-current causes an automatic retrying fault
    AutoRetry = 0b01,
    /// Over-current is reported but no action is taken
    Report = 0b10,
    /// Over-current is not reported and no action is taken
    Ignore = 0b11,
}

/// Deglitch time. Part of the [OcpControl] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum DeglitchTime {
    /// 2 μs deglitch time
    _2 = 0b00,
    /// 4 μs deglitch time
    _4 = 0b01,
    /// 6 μs deglitch time
    _6 = 0b10,
    /// 8 μs deglitch time
    _8 = 0b11,
}

/// V_DS level. Part of the [OcpControl] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 4]
pub enum VdsLevel {
    /// 0.06 V
    _0_06 = 0b0000,
    /// 0.13 V
    _0_13 = 0b0001,
    /// 0.20 V
    _0_20 = 0b0010,
    /// 0.26 V
    _0_26 = 0b0011,
    /// 0.31 V
    _0_31 = 0b0100,
    /// 0.45 V
    _0_45 = 0b0101,
    /// 0.53 V
    _0_53 = 0b0110,
    /// 0.60 V
    _0_60 = 0b0111,
    /// 0.68 V
    _0_68 = 0b1000,
    /// 0.75 V
    _0_75 = 0b1001,
    /// 0.94 V
    _0_94 = 0b1010,
    /// 1.13 V
    _1_13 = 0b1011,
    /// 1.30 V
    _1_30 = 0b1100,
    /// 1.50 V
    _1_50 = 0b1101,
    /// 1.70 V
    _1_70 = 0b1110,
    /// 1.88 V
    _1_88 = 0b1111,
}

/// Over-current protection control register.
#[derive(RWRegister, Clone, Copy)]
#[register(addr = 0x05, ty = "u8")]
#[bitfield(bytes = 2)]
pub struct OcpControl {
    #[skip]
    __: B5,
    /// Time before VDS_OCP and SEN_OCP retries
    pub t_retry: TRetry,
    /// Dead-time insertion
    pub dead_time: DeadTime,
    /// OCP mode
    pub ocp_mode: OcpMode,
    /// OCP deglitch time
    pub ocp_deg: DeglitchTime,
    /// V_DS level
    pub vds_lvl: VdsLevel,
}

impl SerializableRegister<2> for OcpControl {
    fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bytes(bytes)
    }

    fn to_bytes(self) -> [u8; 2] {
        Self::into_bytes(self)
    }
}

/// Current sense amplifier gain. Part of the [CsaControl] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum CsaGain {
    /// 5 V/V
    _5 = 0b00,
    /// 10 V/V
    _10 = 0b01,
    /// 20 V/V
    _20 = 0b10,
    /// 40 V/V
    _40 = 0b11,
}

/// Current sense amplifier over-current protection level. Part of the [CsaControl] register.
#[derive(BitfieldSpecifier, Clone, Copy, Debug)]
#[bits = 2]
pub enum SenLevel {
    /// 0.25 V
    _0_25 = 0b00,
    /// 0.50 V
    _0_50 = 0b01,
    /// 0.75 V
    _0_75 = 0b10,
    /// 1.00 V
    _1_00 = 0b11,
}

/// Current sense amplifier control register.
#[derive(RWRegister, Clone, Copy)]
#[register(addr = 0x06, ty = "u8")]
#[bitfield(bytes = 2)]
pub struct CsaControl {
    #[skip]
    __: B5,
    /// If true, the sense amplifier positive input is SHx, otherwise its SPx.
    pub csa_fet: bool,
    /// Enable dividing the Vref level by 2 (for bi-directional current sense).
    pub vref_div: bool,
    /// If true, VDS_OCP for the low-side MOSFET is measured from SHx to SNx, otherwise from SHx to SPx.
    pub ls_ref: bool,
    /// CSA gain
    pub csa_gain: CsaGain,
    /// Sense over-current fault disable
    pub dis_sen: bool,
    /// Enable calibration on current sense amplifier A
    pub csa_cal_a: bool,
    /// Enable calibration on current sense amplifier B
    pub csa_cal_b: bool,
    /// Enable calibration on current sense amplifier C
    pub csa_cal_c: bool,
    /// Sense over-current protection level
    pub sen_lvl: SenLevel,
}

impl SerializableRegister<2> for CsaControl {
    fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bytes(bytes)
    }

    fn to_bytes(self) -> [u8; 2] {
        Self::into_bytes(self)
    }
}
