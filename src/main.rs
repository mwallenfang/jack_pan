
use itertools::izip;
use ringbuf::RingBuffer;
use std::f32::consts::PI;
use std::io;
use std::str::FromStr;

fn main() {
    // 1. open a client
    let (client, _status) =
        jack::Client::new("jack_pan", jack::ClientOptions::NO_START_SERVER).unwrap();

    // 2. register ports
    let mut out_port_l = client
        .register_port("pan_out_l", jack::AudioOut::default())
        .unwrap();

    let mut out_port_r = client
        .register_port("pan_out_r", jack::AudioOut::default())
        .unwrap();

    let in_port_l = client
        .register_port("pan_in_l", jack::AudioIn::default())
        .unwrap();

    let in_port_r = client
        .register_port("pan_in_r", jack::AudioIn::default())
        .unwrap();

    // 3. define process callback handler
    let rb = RingBuffer::<f32>::new(client.sample_rate());
    let (mut prod, mut cons) = rb.split();

    // The pan variable, currently
    // 0 - left
    // 0.5 - middle
    // 1 - right
    let mut pan = 0.5;

    // Define the amount of steps to be the amount of samples in 50ms
    let process = jack::ClosureProcessHandler::new(
        move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            // Get output buffer
            let out_p_l = out_port_l.as_mut_slice(ps);
            let out_p_r = out_port_r.as_mut_slice(ps);

            let in_p_l = in_port_l.as_slice(ps);
            let in_p_r = in_port_r.as_slice(ps);

            if let Some(f) = cons.pop() {
                pan = f;
            }

            // Write output
            for (input_l, input_r, output_l, output_r) in izip!(in_p_l, in_p_r, out_p_l, out_p_r) {
                // Calculate the side gains for the given pan law and apply it
                let pan_gain = db4_5_pan(pan);
                *output_l = pan_gain.0 * input_l;
                *output_r = pan_gain.1 * input_r;
            }

            // Continue as normal
            jack::Control::Continue
        },
    );

    // 4. Activate the client. Also connect the ports to the system audio.
    let _active_client = client.activate_async((), process).unwrap();

    // processing starts here

    // 5. wait or do some processing while your handler is running in real time.
    loop {
        if let Some(f) = read_freq() {
            prod.push(f).unwrap();
        }
    }
    // 6. Optional deactivate. Not required since active_client will deactivate on
    // drop, though explicit deactivate may help you identify errors in
    // deactivate.
    //_active_client.deactivate().unwrap();
}

/// Attempt to read a frequency from standard in. Will block until there is
/// user input. `None` is returned if there was an error reading from standard
/// in, or the retrieved string wasn't a compatible u16 integer.
fn read_freq() -> Option<f32> {
    let mut user_input = String::new();
    match io::stdin().read_line(&mut user_input) {
        Ok(_) => f32::from_str(user_input.trim()).ok(),
        Err(_) => None,
    }
}

#[inline]
/// Converts a db change into a linear value
///
/// Source: https://mu.krj.st/mix/
///
fn db2lin(input: f32) -> f32 {
    10.0_f32.powf(input * 0.05) as f32
}

/// Calculates the linear gain of the left and right channel given the input pan_factor
///
/// This is a linear calculation, which is normally not used, since it doesn't sound that good,
/// since it has a "hole" in the middle
fn linear_pan(factor: f32) -> (f32, f32) {
    (factor, (1.0 - factor))
}

/// Calculates the linear gain of the left and right channel given the input pan_factor
/// using the constant power pan
///
/// factor in [0,1] 0 : left, 1 : right
/// This uses the formula:
/// left_gain = cos(factor * (pi/2))
/// right_gain = sin(factor * (pi/2))
///
/// It is still a little bit quieter in the middle, but not as much as the linear pan
/// Source: https://www.kvraudio.com/forum/viewtopic.php?t=148865
fn constant_power_pan(factor: f32) -> (f32, f32) {
    (
        (factor * (PI / 2.0)).cos(),
        (factor * (PI / 2.0)).sin()
    )
}

/// Calculates the linear gain of the left and right channel given the input pan_factor
/// using the -4.5 dB Pan Law
///
/// factor in [0,1] 0 : left, 1 : right
/// This uses the formula:
/// left_gain = sqrt[((PI / 2.0) - factor * (PI / 2.0)) * (2.0/PI) * cos(factor * (PI / 2.0))]
/// right_gain = sqrt[factor * (PI / 2.0) * (2.0/PI) * sin(factor * (PI / 2.0))]
///
/// This is also a little bit quieter in the middle, even a bit quieter than constant power pan
/// Source: https://www.cs.cmu.edu/~music/icm-online/readings/panlaws/index.html
fn db4_5_pan(factor: f32) -> (f32, f32) {
    (
        (((PI / 2.0) - factor * (PI / 2.0)) * (2.0/PI) * (factor * (PI / 2.0)).cos()).sqrt(),
        (factor * (PI / 2.0) * (2.0/PI) * (factor * (PI / 2.0)).sin()).sqrt()
    )
}
