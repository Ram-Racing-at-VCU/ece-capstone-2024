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

    let sector = match angle {
        angle if 0. < angle && angle <= PI / 3. => 1,
        angle if PI / 3. < angle && angle <= 2. * PI / 3. => 2,
        angle if 2. * PI / 3. < angle && angle <= PI => 3,
        angle if PI < angle && angle <= 4. * PI / 3. => 4,
        angle if 4. * PI / 3. < angle && angle <= 5. * PI / 3. => 5,
        angle if 5. * PI / 3. < angle && angle <= 2. * PI => 6,
        _ => {
            panic!("Invalid sector!");
        }
    };

    // defmt::info!("angle = {}", angle);
    // defmt::info!("sector = {}", sector);

    let t1: f32 = modulation_index * ts * ((sector as f32) * PI / 3. - angle).sin();
    let t2: f32 = modulation_index * ts * (angle - ((sector - 1) as f32) * PI / 3.).sin();
    let t0: f32 = ts - t1 - t2;

    let t_a: f32;
    let t_b: f32;
    let t_c: f32;

    match sector {
        1 => {
            t_a = t0 * 0.5 + t1 + t2;
            t_b = t0 * 0.5 + t2;
            t_c = t0 * 0.5;
        }

        2 => {
            t_a = t0 * 0.5 + t1;
            t_b = t0 * 0.5 + t1 + t2;
            t_c = t0 * 0.5;
        }

        3 => {
            t_a = t0 * 0.5;
            t_b = t0 * 0.5 + t1 + t2;
            t_c = t0 * 0.5 + t2;
        }

        4 => {
            t_a = t0 * 0.5;
            t_b = t0 * 0.5 + t1;
            t_c = t0 * 0.5 + t1 + t2;
        }

        5 => {
            t_a = t0 * 0.5 + t2;
            t_b = t0 * 0.5;
            t_c = t0 * 0.5 + t1 + t2;
        }

        6 => {
            t_a = t0 * 0.5 + t1 + t2;
            t_b = t0 * 0.5;
            t_c = t0 * 0.5 + t1;
        }

        _ => {
            defmt::panic!("Invalid Sector!")
        }
    }

    (t_a, t_b, t_c)
}
