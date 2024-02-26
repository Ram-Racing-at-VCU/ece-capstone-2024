use control_algorithms::PIDController;

fn main() {
    // let mut controller = PIDController::default();
    let mut controller = PIDController::new(1., 1., 1., 0.001);
    let input_signal: Vec<f32> = vec![1.; 1000];
    let measurement: f32 = 0.;
    let mut output_signal: Vec<f32> = Vec::new();

    for input in input_signal {
        output_signal.push(controller.update(input, measurement));
    }

    for i in output_signal {
        println!("{}", i)
    }
}
