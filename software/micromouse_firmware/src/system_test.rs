use core::fmt::Write;
use core::str;

use embedded_hal::blocking::i2c;
use embedded_hal::digital::v2::{InputPin, OutputPin, ToggleableOutputPin};

use crate::battery::Battery;
use crate::motors::left::{LeftEncoder, LeftMotor};
use crate::motors::right::{RightEncoder, RightMotor};
use crate::motors::{Encoder, Motor};
use crate::time::Time;
use crate::uart::Uart;
use crate::vl6180x::VL6180x;

struct MotorCommand<M: Motor, E: Encoder> {
    pub motor: M,
    pub encoder: E,
    pub reporting: bool,
}

impl<M: Motor, E: Encoder> MotorCommand<M, E> {
    pub fn parse<'a, I: Iterator<Item = &'a str>>(
        &mut self,
        uart: &mut Uart,
        mut words: I,
    ) {
        match words.next() {
            Some("report") => match words.next() {
                Some("on") => self.reporting = true,
                Some("off") => self.reporting = false,
                word => {
                    writeln!(uart, "Unknown command: {:?}", word).ok();
                }
            },
            Some("set") => {
                if let Some(power) = words.next().and_then(|word| word.parse().ok()) {
                    self.motor.change_power(power);
                } else {
                    writeln!(uart, "Expected a number").ok();
                }
            }
            word => {
                writeln!(uart, "Unknown command: {:?}", word).ok();
            }
        }
    }

    pub fn report(&self, uart: &mut Uart, name: &str) {
        if self.reporting {
            write!(uart, "{}:{},", name, self.encoder.count()).ok();
        }
    }
}

/// Allows testing of the mouse hardware over UART
pub fn do_system_test<RL, GL, BL, OL, LB, RB, I2C1, I2C2, I2C3>(
    mut time: Time,
    battery: Battery,
    _red_led: RL,
    _green_led: GL,
    _blue_led: BL,
    _orange_led: OL,
    _left_button: LB,
    _right_button: RB,
    mut left_motor: LeftMotor,
    mut right_motor: RightMotor,
    left_encoder: LeftEncoder,
    right_encoder: RightEncoder,
    _front_distance: VL6180x<I2C1>,
    _left_distance: VL6180x<I2C2>,
    _right_distance: VL6180x<I2C3>,
    mut uart: Uart,
) -> !
where
    RL: OutputPin + ToggleableOutputPin,
    GL: OutputPin + ToggleableOutputPin,
    BL: OutputPin + ToggleableOutputPin,
    OL: OutputPin + ToggleableOutputPin,
    LB: InputPin,
    RB: InputPin,
    I2C1: i2c::Read + i2c::Write + i2c::WriteRead,
    I2C2: i2c::Read + i2c::Write + i2c::WriteRead,
    I2C3: i2c::Read + i2c::Write + i2c::WriteRead,
{
    let mut time_report = false;

    let mut left_motor_command = MotorCommand {
        motor: left_motor,
        encoder: left_encoder,
        reporting: false,
    };

    let mut right_motor_command = MotorCommand {
        motor: right_motor,
        encoder: right_encoder,
        reporting: false,
    };

    let mut last_time = 0;

    loop {
        if let Some(buf) = uart.read_line().ok() {
            if let Some(line) = str::from_utf8(&buf).ok() {
                let mut words = line.split_whitespace();
                match words.next() {
                    Some("time") => match words.next() {
                        Some("report") => match words.next() {
                            Some("on") => time_report = true,
                            Some("off") => time_report = false,
                            word => {
                                writeln!(uart, "Unknown command: {:?}", word);
                            }
                        },
                        word => {
                            writeln!(uart, "Unknown command: {:?}", word);
                        }
                    },
                    Some("motor") => match words.next() {
                        Some("left") => left_motor_command.parse(&mut uart, words),
                        Some("right") => right_motor_command.parse(&mut uart, words),
                        word => {
                            writeln!(uart, "Unknown command: {:?}", word);
                        }
                    },
                    word => {
                        writeln!(uart, "Unknown command: {:?}", word);
                    }
                }
            }
        }

        if time.now() - last_time >= 10 {
            if time_report {
                write!(uart, "T:{},", time.now()).ok();
            }

            left_motor_command.report(&mut uart, "LM");
            right_motor_command.report(&mut uart, "LM");

            if time_report
                || left_motor_command.reporting
                || right_motor_command.reporting
            {
                uart.add_str("\n");
            }

            last_time = time.now();
        }
    }
}
