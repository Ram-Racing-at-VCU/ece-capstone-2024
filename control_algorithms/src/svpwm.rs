//! SVPWM implementation
use core::f32::consts::PI;
use micromath::F32Ext;

/// todo: documentation
pub fn svpwm(v_alpha: f32, v_beta: f32, angle: f32, v_dc: f32, ts: f32) -> (f32, f32, f32) {
    let v_reference: f32 = (v_alpha.powi(2) + v_beta.powi(2)).sqrt();
    let modulation_index: f32 = (v_reference * PI / 2.) / v_dc;
    let mut phase: f32 = angle;

    let mut sector: usize = 1;
    while phase > PI / 3. {
        phase -= PI / 3.;
        sector += 1;
    }

    let t1: f32 = modulation_index * (angle.cos() - (1. / 3.0.sqrt()) * angle.sin());
    let t2: f32 = modulation_index * (2. / 3.0.sqrt()) * angle.sin();
    let t0: f32 = 1.0 - t1 - t2;

    let t_a: f32;
    let t_b: f32;
    let t_c: f32;

    match sector {
        1 => {
            t_a = ts * (0.5 * t0);
            t_b = ts * (0.5 * t0 + t1);
            t_c = ts * (1.0 - 0.5 * t0);
        }

        2 => {
            t_a = ts * (0.5 * t0 + t2);
            t_b = ts * 0.5 * t0;
            t_c = ts * (1.0 - 0.5 * t0);
        }

        3 => {
            t_a = ts * (1.0 - 0.5 * t0);
            t_b = ts * 0.5 * t0;
            t_c = ts * (0.5 * t0 + t1);
        }

        4 => {
            t_a = ts * (1.0 - 0.5 * t0);
            t_b = ts * (0.5 * t0 + t2);
            t_c = ts * t0 * 0.5;
        }

        5 => {
            t_a = ts * (0.5 * t0 + t1);
            t_b = ts * (1.0 - 0.5 * t0);
            t_c = ts * t0 * 0.5;
        }

        6 => {
            t_a = ts * t0 * 0.5;
            t_b = ts * (1.0 - 0.5 * t0);
            t_c = ts * (0.5 * t0 + t2);
        }

        _ => {
            panic!("Invalid Sector!")
        }
    }

    (t_a, t_b, t_c)
}
