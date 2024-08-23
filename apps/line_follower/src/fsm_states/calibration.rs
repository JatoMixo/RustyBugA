/// Calibration state
///
/// The calibration state is the state where the line follower is in when the user wants to calibrate
/// the line follower. Here the line follower waits for the user to press the button 1 to start the
/// calibration process.
///
/// Once the calibration is done, the line follower beeps and waits for the user inputs.
///
/// The user can also use the serial rx to send the button 1 or button 2 command to the line follower.
///
/// The state output events are:
/// - Button1Pressed: When the user presses the button 1.
/// - Button2Pressed: When the user presses the button 2.
/// - BatteryIsLow: When the battery is low.
///
use crate::board::timer::SysDelay;
use hal_button::ButtonController;
use mightybuga_bsc::prelude::*;
use mightybuga_bsc::timer_based_buzzer::TimerBasedBuzzer;
use mightybuga_bsc::timer_based_buzzer::TimerBasedBuzzerInterface;
use mightybuga_bsc::light_sensor_array::LightSensorArray;
use light_sensor_array_controller::LightSensorArrayController;
use engine::engine::EngineController;

use crate::fsm::FSMEvent;
use crate::line_follower_status::LineFollowerStatus;

use logging::Logger;

pub fn run(status: &mut LineFollowerStatus) -> FSMEvent {
    let mut logger = Logger::new(&mut status.board.serial.tx);
    logger.log("Calibration state\r\n");

    logger.log("Press button 1 to start calibration\r\n");

    // Wait for buttons to be released before waiting for buttons to be pressed
    status.board.delay.delay_ms(2000u32);

    loop {
        if status.board.btn_1.is_pressed() {
            break;
        }

        if status.board.btn_2.is_pressed() {
            logger.log("Exit calibration\r\n");
            return FSMEvent::Button2Pressed;
        }

        if let Ok(serial_input) = status.board.serial.rx.read() {
            match serial_input {
                b'1' => break,
                b'2' => {
                    logger.log("Exit calibration\r\n");
                    return FSMEvent::Button2Pressed;
                }
                _ => {}
            }
        }
    }

    logger.log("Calibration started\r\n");

    status.board.light_sensor_array.set_led(true);

    let mut min_values = [0; 8];
    let mut max_values = [0; 8];

    // For calibrating the sensor, we choose a direction and rotate until the line is in the
    // opposite light sensor, then we rotate in the other direction and repeat. While we move, we
    // get the values from the light sensor array and store the maximums and minimums to then
    // calculate the threshold where the line can be found.

    let mut direction = RotationDirection::Left;
    for _ in 0..6 {

        let light_values = status.board.light_sensor_array.get_light_map();

        loop {
            rotate_direction(&mut status.board.engine, &mut status.board.delay, &direction);
            update_min_and_max_values(&light_values, &mut min_values, &mut max_values);

            if get_min_value_of_array(&light_values) == light_values[0] {
                direction = RotationDirection::Left;
                break;
            }

            if get_min_value_of_array(&light_values) == light_values[7] {
                direction = RotationDirection::Right;
                break;
            }
        }
    }

    status.light_sensor_thresholds = Some(calculate_light_thresholds(max_values, min_values));

    status.board.light_sensor_array.set_led(false);
    status.board.delay.delay_ms(3000u32);

    logger.log("Calibration done\r\n");

    beep(&mut status.board.buzzer, &mut status.board.delay);
    status.board.delay.delay_ms(50u32);
    beep(&mut status.board.buzzer, &mut status.board.delay);

    logger.log("Press button 1 to start line following\r\n");
    logger.log("Press button 2 to go back to idle\r\n");
    loop {
        if status.board.btn_1.is_pressed() {
            return FSMEvent::Button1Pressed;
        }
        if status.board.btn_2.is_pressed() {
            logger.log("Exit calibration\r\n");
            return FSMEvent::Button2Pressed;
        }
        if let Ok(serial_input) = status.board.serial.rx.read() {
            match serial_input {
                b'1' => return FSMEvent::Button1Pressed,
                b'2' => return FSMEvent::Button2Pressed,
                _ => {}
            }
        }
    }
}

fn get_min_value_of_array(array: &[u16; 8]) -> u16 {
    let mut min = u16::MAX;

    for &element in array {
        if element < min {
            min = element;
        }
    }

    min
}

enum RotationDirection {
    Left,
    Right,
}

fn update_min_and_max_values(
    light_values: &[u16; 8],
    min_values: &mut [u16; 8],
    max_values: &mut [u16; 8]
) {
    light_values.iter().enumerate().for_each(|(index, &light_value)| {
        if light_value > max_values[index] {
            max_values[index] = light_value;
        }

        if light_value < min_values[index] {
            min_values[index] = light_value;
        }
    });
}

fn rotate_direction<T: EngineController>(
    engine: &mut T,
    delay: &mut SysDelay,
    direction: &RotationDirection
) {
    // TODO: Get proper values for this
    const ROTATION_DUTY: u16 = u16::MAX / 5;
    const ROTATION_TIME_MS: u32 = 50u32;

    match &direction {
        RotationDirection::Left => {
            engine.rotate_left(ROTATION_DUTY);
        },
        RotationDirection::Right => {
            engine.rotate_right(ROTATION_DUTY);
        }
    }


    delay.delay_ms(ROTATION_TIME_MS);
    engine.stop();
}

fn calculate_light_thresholds(
    max_values: [u16; 8],
    min_values: [u16; 8]
) -> [u16; 8] {
    let mut thresholds: [u16; 8] = [0; 8];

    max_values.iter().enumerate().for_each(|(index, &max_value)| {
        let min_value = min_values[index];

        thresholds[index] = (max_value + min_value) / 2;
    });

    thresholds
}

fn beep(buzzer: &mut TimerBasedBuzzer, delay: &mut SysDelay) {
    buzzer.turn_on();
    buzzer.change_frequency(70, 1828);
    delay.delay_ms(100u32);
    buzzer.turn_off();
}
