use core::convert::Infallible;

use defmt::error;
use drv8323rs::{registers::*, Drv8323rs, ReadRegister, WriteRegister};

use embassy_embedded_hal::shared_bus::{asynch::spi::SpiDevice, SpiDeviceError};
use embassy_stm32::{
    gpio::Output,
    mode::Async,
    peripherals::SPI1,
    spi::{Error as SpiError, Spi},
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;

use super::consts::DRV_INIT_READS;

type Drv<'a> = Drv8323rs<SpiDevice<'a, NoopRawMutex, Spi<'static, SPI1, Async>, Output<'static>>>;

type Error = SpiDeviceError<SpiError, Infallible>;

/// Repeatedly reads from one of the configuration registers of the DRV
/// to ensure that the device is functional.
pub async fn check_driver(drv: &mut Drv<'_>) -> Result<bool, Error> {
    let mut failures = 0;
    for _ in 0..DRV_INIT_READS {
        let register: CsaControl = drv.read().await?;
        if register.into_bytes() != [0x02, 0x83] {
            failures += 1;
        }
    }

    if failures != 0 {
        error!(
            "DRV didn't function correctly ({}% failure rate)",
            failures as f32 * 100.0 / DRV_INIT_READS as f32
        );
        Ok(false)
    } else {
        Ok(true)
    }
}

/// Set up the DRV8323RS configuration.
///
/// All values are calculated based on our custom hardware.
pub async fn setup_driver(drv: &mut Drv<'_>) -> Result<(), Error> {
    /* Drive Control Register */

    // OTW reporting is turned on, everything else is default

    let drive_control = DriveControl::new()
        .with_dis_cpuv(false)
        .with_dis_gdf(false)
        .with_otw_rep(true)
        .with_pwm_mode(PwmMode::_6x)
        .with_single_pwm_com(false)
        .with_single_pwm_dir(false)
        .with_coast(false)
        .with_brake(false)
        .with_clr_flt(false);

    drv.write(drive_control).await?;

    /* Gate High Side Register */

    // rise time ~ 124 ns
    // fall time ~  62 ns

    let gate_hs = GateHs::new()
        .with_lock(Lock::Unlock)
        .with_idrivep_hs(SourceCurrent::_80)
        .with_idriven_hs(SinkCurrent::_160);

    drv.write(gate_hs).await?;

    /* Gate Low Side Register */

    // note: we are using latched OCP shutdown so CBC isn't in use
    // rise and fall times are the same as the high-side

    let gate_ls = GateLs::new()
        .with_cbc(false)
        .with_t_drive(TDrive::_500) // there is no smaller value
        .with_idrivep_ls(SourceCurrent::_80)
        .with_idriven_ls(SinkCurrent::_160);

    drv.write(gate_ls).await?;

    /* Over Current Protection Control Register */

    // - using latched shutdown for safety while testing control algorithms
    // - vds level is based on datasheet, 0.06 V would be a more conservative value
    // - dead time is the default value

    let ocp_control = OcpControl::new()
        .with_t_retry(TRetry::_4ms)
        .with_dead_time(DeadTime::_100)
        .with_ocp_mode(OcpMode::Latched)
        .with_ocp_deg(DeglitchTime::_4)
        .with_vds_lvl(VdsLevel::_0_13);

    drv.write(ocp_control).await?;

    /* Current Sense Amplifier Control Register */

    // - current sense resistor is from SPA - SNA
    // - max safe gain is 10 V/V
    // - sense OCP level should be 2mOhm * 80A = 0.16V, but 0.25V is the lowest possible

    let csa_control = CsaControl::new()
        .with_csa_fet(false)
        .with_vref_div(true)
        .with_ls_ref(false)
        .with_csa_gain(CsaGain::_10)
        .with_dis_sen(false)
        .with_csa_cal_a(false)
        .with_csa_cal_b(false)
        .with_csa_cal_c(false)
        .with_sen_lvl(SenLevel::_0_25);

    drv.write(csa_control).await?;

    Ok(())
}

/// Print error messages for every latched bit of the status registers
pub async fn report_status(drv: &mut Drv<'_>) -> Result<(), Error> {
    let status_1: Status1 = drv.read().await?;

    if status_1.vds_ocp() {
        error!("DRV Fault: VDS over-current protection");
    }

    if status_1.gdf() {
        error!("DRV Fault: gate drive fault");
    }

    if status_1.uvlo() {
        error!("DRV Fault: under-voltage lockout");
    }

    if status_1.otsd() {
        error!("DRV Fault: over-temperature shut-down");
    }

    if status_1.vds_ha() {
        error!("DRV Fault: phase A high-side MOSFET");
    }

    if status_1.vds_la() {
        error!("DRV Fault: phase A low-side MOSFET");
    }

    if status_1.vds_hb() {
        error!("DRV Fault: phase B high-side MOSFET");
    }

    if status_1.vds_lb() {
        error!("DRV Fault: phase B low-side MOSFET");
    }

    if status_1.vds_hc() {
        error!("DRV Fault: phase C high-side MOSFET");
    }

    if status_1.vds_lc() {
        error!("DRV Fault: phase C low-side MOSFET");
    }

    let status_2: Status2 = drv.read().await?;

    if status_2.sa_oc() {
        error!("DRV Fault: over-current on phase A sense amplifier");
    }

    if status_2.sb_oc() {
        error!("DRV Fault: over-current on phase B sense amplifier");
    }

    if status_2.sc_oc() {
        error!("DRV Fault: over-current on phase C sense amplifier");
    }

    if status_2.otw() {
        error!("DRV Fault: over-temperature warning");
    }

    if status_2.cp_uv() {
        error!("DRV Fault: charge pump under-voltage");
    }

    if status_2.vgs_ha() {
        error!("DRV Fault: gate drive fault on phase A high-side MOSFET");
    }

    if status_2.vgs_la() {
        error!("DRV Fault: gate drive fault on phase A low-side MOSFET");
    }

    if status_2.vgs_hb() {
        error!("DRV Fault: gate drive fault on phase B high-side MOSFET");
    }

    if status_2.vgs_lb() {
        error!("DRV Fault: gate drive fault on phase B low-side MOSFET");
    }

    if status_2.vgs_hc() {
        error!("DRV Fault: gate drive fault on phase C high-side MOSFET");
    }

    if status_2.vgs_lc() {
        error!("DRV Fault: gate drive fault on phase C low-side MOSFET");
    }

    Ok(())
}
