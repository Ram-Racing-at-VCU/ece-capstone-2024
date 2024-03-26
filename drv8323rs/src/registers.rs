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

/// Available PWM modes on the DRV8323RS. Part of the `Control` register.
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

/// The status registers are used to reporting warning and fault conditions.
#[derive(RWRegister, Clone, Copy)]
#[register(addr = 0x02, ty = "u8")]
#[bitfield(bytes = 2)]
pub struct Control {
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

impl SerializableRegister<2> for Control {
    fn from_bytes(bytes: [u8; 2]) -> Self {
        Self::from_bytes(bytes)
    }

    fn to_bytes(self) -> [u8; 2] {
        Self::into_bytes(self)
    }
}

// #[derive(RWRegister)]
// #[register(addr = 0x03, ty = "u8")]
// #[bitfield(bytes = 2)]

// pub struct GateDriveHS {
//     pub lock: bool,
//     /// Gate drive peak source current
//     pub idrivep_hs: bool,
//     /// Gate drive peak sink current
//     pub idriven_hs: bool,
//     /// Gate drive peak source current

// }

// impl SerializableRegister<2> for GateDriveHS {
//     fn from_bytes(bytes: [u8; 2]) -> Self {
//         Self::from_bytes(bytes)
//     }

//     fn to_bytes(self) -> [u8; 2] {
//         Self::into_bytes(self)
//     }
// }

// //
// #[derive(RWRegister)]
// #[register(addr = 0x04, ty = "u8")]
// #[bitfield(bytes = 2)]

// pub struct GateDriveLS {
//     /// Gate drive peak source current
//     pub cbc: bool,
//     /// Gate drive peak sink current

//     pub idrivep_ls: bool,
//     /// Gate drive peak sink current
//     pub idriven_ls: bool,
//     /// Gate drive peak source current
//     pub tdrive: bool,
//     /// Gate drive peak sink current

// }

// impl SerializableRegister<2> for GateDriveLS {
//     fn from_bytes(bytes: [u8; 2]) -> Self {
//         Self::from_bytes(bytes)
//     }

//     fn to_bytes(self) -> [u8; 2] {
//         Self::into_bytes(self)
//     }
// }

// ///
// #[derive(RWRegister)]
// #[register(addr = 0x05, ty = "u8")]
// #[bitfield(bytes = 2)]
// pub struct OCP {
//     #[skip]
//     __: B3,
//     /// Over-current protection mode
//     pub tretry: bool,
//     pub dead_time: bool,
//     pub  ocp_mode : bool,
//     pub ocp_deg: bool,
//     pub vds_lvl: bool,

// }
// // impl SerializableRegister<2> for OCP {
// //     fn from_bytes(bytes: [u8; 2]) -> Self {
// //         Self::from_bytes(bytes)
// //     }

// //     fn to_bytes(self) -> [u8; 2] {
// //         Self::into_bytes(self)
// //     }
// // }

// ///
// #[derive(RWRegister)]
// #[register(addr = 0x06, ty = "u8")]
// #[bitfield(bytes = 2)]

// pub struct CSA {
//     #[skip]
//     __: B7,
//     pub csa_fet: bool,
//     pub vref_div: bool,
//     pub ls_ref: bool,
//     pub csa_gain: bool,
//     pub dis_sen: bool,
//     pub csa_cal_a: bool,
//     pub csa_cal_b: bool,
//     pub csa_cal_c: bool,
//     pub sen_lvl: bool,

// }

// impl SerializableRegister<2> for CSA {
//     fn from_bytes(bytes: [u8; 2]) -> Self {
//         Self::from_bytes(bytes)
//     }

//     fn to_bytes(self) -> [u8; 2] {
//         Self::into_bytes(self)
//     }
// }
