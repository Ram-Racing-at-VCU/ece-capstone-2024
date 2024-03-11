//! SVPWM implementation
use core::f32::consts::PI;
use defmt;
use micromath::F32Ext;

/// todo: documentation
pub fn svpwm(v_alpha: f32, v_beta: f32, angle: f32, v_dc: f32, ts: f32) -> (f32, f32, f32) {
    let v_reference: f32 = (v_alpha * v_alpha + v_beta * v_beta).sqrt();
    if v_reference > v_dc {
        panic!("Error: Reference Voltage cannot be greater than DC Link Voltage!")
    }
    let modulation_index: f32 = v_reference / v_dc;
    let mut phase: f32 = angle;

    let mut sector: usize = 1;
    while phase > PI / 3. {
        phase -= PI / 3.;
        sector += 1;
    }

    defmt::info!("Sector: {}", sector);

    let t1: f32 = 3.0.sqrt() * modulation_index * ts * (PI / 3. - angle).sin();
    let t2: f32 = modulation_index * ts * 3.0.sqrt() * angle.sin();
    let t0: f32 = (ts - t1 - t2) / 2.;

    let t_a: f32;
    let t_b: f32;
    let t_c: f32;

    match sector {
        1 => {
            t_a = 0.5 * t0;
            t_b = 0.5 * t0 + t1;
            t_c = 0.5 * t0 + t1 + t2;
        }

        2 => {
            t_a = 0.5 * t0 + t2;
            t_b = 0.5 * t0;
            t_c = t1 + t2 + t0 * 0.5;
        }

        3 => {
            t_a = t1 + t2 + t0 * 0.5;
            t_b = 0.5 * t0;
            t_c = 0.5 * t0 + t1;
        }

        4 => {
            t_a = t1 + t2 + t0 * 0.5;
            t_b = 0.5 * t0 + t2;
            t_c = t0 * 0.5;
        }

        5 => {
            t_a = 0.5 * t0 + t1;
            t_b = t1 + t2 + t0 * 0.5;
            t_c = t0 * 0.5;
        }

        6 => {
            t_a = t0 * 0.5;
            t_b = t1 + t2 + t0 * 0.5;
            t_c = 0.5 * t0 + t2;
        }

        _ => {
            defmt::panic!("Invalid Sector!")
        }
    }

    (t_a, t_b, t_c)
}
