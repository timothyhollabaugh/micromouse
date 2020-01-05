#![no_std]
#![no_main]

/**
 *
 * Important mechanical parameters:
 *
 *  - wheel diameter: 32mm
 *  - wheel circumference: 100mm
 *  - gear raitio: 75:1
 *  - counts per motor rev: 12
 *  - counts per wheel rev: 900
 *  - counts per mm: 9
 *  - wheelbase diameter: 73mm
 *  - wheelbase circumference: 229.336mm
 *  - ticks per spin: 2064.03
 *
 *  Positive spin is clockwise (right)
 *  Positive linear is forward
 *
 */
// pick a panicking behavior
// you can put a breakpoint on `rust_begin_unwind` to catch panics
extern crate panic_halt;

pub mod battery;
pub mod motors;
pub mod time;
pub mod uart;
pub mod vl6180x;

use core::fmt::Write;
use cortex_m_rt::entry;
use cortex_m_rt::exception;
use cortex_m_rt::ExceptionFrame;
use stm32f4xx_hal as stm32f4;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::stm32 as stm32f405;

use serde::Serialize;

use heapless::Vec;

use postcard;

use typenum::consts::U1024;

use mouse::comms::DebugMsg;
use mouse::comms::DebugPacket;
#[allow(unused_imports)]
use mouse::config::*;
use mouse::map::Direction;
use mouse::map::Orientation;
use mouse::map::Vector;
use mouse::mouse::Mouse;
use mouse::mouse::MouseConfig;
use mouse::mouse::MouseDebug;

use crate::battery::Battery;
use crate::time::Time;

use crate::uart::Uart;

use crate::motors::{Encoder, Motor};

use crate::motors::left::{LeftEncoder, LeftMotor};
use crate::motors::right::{RightEncoder, RightMotor};

// Setup the master clock out
pub fn mco2_setup(rcc: &stm32f405::RCC, gpioc: &stm32f405::GPIOC) {
    rcc.ahb1enr.write(|w| w.gpiocen().set_bit());
    rcc.cfgr.modify(|_, w| w.mco2().sysclk());
    gpioc.moder.write(|w| w.moder9().alternate());
    gpioc.afrh.write(|w| w.afrh9().af0());
}

#[entry]
fn main() -> ! {
    let p = stm32f4::stm32::Peripherals::take().unwrap();
    let _cp = stm32f405::CorePeripherals::take().unwrap();

    // Init non-hal things
    let mut time = Time::setup(&p.RCC, p.TIM1);

    while time.now() < 10000 {}

    let mut battery = Battery::setup(&p.RCC, &p.GPIOB, p.ADC1);

    let mut uart = Uart::setup(&p.RCC, p.USART1, &p.GPIOA);

    let mut left_motor = LeftMotor::setup(&p.RCC, p.TIM3, &p.GPIOA);
    let left_encoder = LeftEncoder::setup(&p.RCC, &p.GPIOA, &p.GPIOB, p.TIM2);

    let mut right_motor = RightMotor::setup(&p.RCC, p.TIM4, &p.GPIOB);
    let right_encoder = RightEncoder::setup(&p.RCC, &p.GPIOA, p.TIM5);

    // Init the hal things
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();

    let mut red_led = gpiob.pb12.into_push_pull_output();
    let mut green_led = gpiob.pb13.into_push_pull_output();
    let mut blue_led = gpiob.pb14.into_push_pull_output();
    let mut orange_led = gpiob.pb15.into_push_pull_output();

    let left_button = gpioc.pc10.into_pull_up_input();
    let _middle_button = gpioc.pc11.into_pull_up_input();
    let right_button = gpioc.pc12.into_pull_up_input();

    orange_led.set_high().ok();
    blue_led.set_low().ok();

    uart.add_bytes(b"Initializing\n").ok();

    let mut front_distance = {
        let scl = gpiob.pb8.into_open_drain_output().into_alternate_af4();
        let sda = gpiob.pb9.into_open_drain_output().into_alternate_af4();

        let mut gpio0 = gpioc.pc0.into_open_drain_output();
        gpio0.set_high().ok();

        let mut gpio1 = gpioc.pc1.into_open_drain_output();
        gpio1.set_high().ok();

        let i2c =
            stm32f4::i2c::I2c::i2c1(p.I2C1, (scl, sda), 100.khz(), clocks);

        time.delay(10000);

        let mut distance = vl6180x::VL6180x::new(i2c, 0x29);
        distance.init_private_registers();
        distance.init_default();
        distance
    };

    orange_led.set_low().ok();
    blue_led.set_high().ok();

    let mut left_distance = {
        let scl = gpiob.pb10.into_open_drain_output().into_alternate_af4();
        let sda = gpiob.pb11.into_open_drain_output().into_alternate_af4();

        let mut gpio0 = gpioc.pc2.into_open_drain_output();
        gpio0.set_high().ok();

        let mut gpio1 = gpioc.pc3.into_open_drain_output();
        gpio1.set_high().ok();

        let i2c =
            stm32f4::i2c::I2c::i2c2(p.I2C2, (scl, sda), 100.khz(), clocks);

        time.delay(1000);

        let mut distance = vl6180x::VL6180x::new(i2c, 0x29);
        distance.init_private_registers();
        distance.init_default();
        distance
    };

    orange_led.set_high().ok();
    blue_led.set_high().ok();

    let mut right_distance = {
        let scl = gpioa.pa8.into_open_drain_output().into_alternate_af4();
        let sda = gpioc.pc9.into_open_drain_output().into_alternate_af4();

        let mut gpio0 = gpioc.pc4.into_open_drain_output();
        gpio0.set_high().ok();

        let mut gpio1 = gpioc.pc5.into_open_drain_output();
        gpio1.set_high().ok();

        let i2c =
            stm32f4::i2c::I2c::i2c3(p.I2C3, (scl, sda), 100.khz(), clocks);

        time.delay(1000);

        let mut distance = vl6180x::VL6180x::new(i2c, 0x29);
        distance.init_private_registers();
        distance.init_default();
        distance
    };

    blue_led.set_low().ok();
    orange_led.set_low().ok();

    let config = MouseConfig {
        mechanical: MOUSE_2019_MECH,
        path: MOUSE_2019_PATH,
        map: MOUSE_MAZE_MAP,
        motion: MOUSE_2019_MOTION,
    };

    let initial_orientation = Orientation {
        position: Vector {
            x: 1250.0,
            y: 1350.0,
        },
        direction: Direction::from(0.0),
    };

    uart.add_bytes(b"\n\nstart").ok();

    let mut last_time: u32 = time.now();

    let mut mouse = Mouse::new(
        &config,
        initial_orientation,
        last_time,
        left_encoder.count(),
        right_encoder.count(),
    );

    front_distance.start_ranging();
    left_distance.start_ranging();
    right_distance.start_ranging();

    let mut running = false;
    let mut debugging = false;

    let mut packet_count = 0;
    let mut step_count = 0;

    loop {
        let now: u32 = time.now();

        front_distance.update();
        right_distance.update();
        left_distance.update();

        if let Ok(byte) = uart.read_byte() {
            match byte {
                0 => {}
                1 => debugging = false,
                2 => debugging = true,
                3 => running = false,
                4 => running = true,
                _ => {}
            }
        }

        if now - last_time >= 10 {
            green_led.toggle().ok();

            if running {
                let left_encoder_count = left_encoder.count();
                let right_encoder_count = right_encoder.count();
                let left_distance_range = left_distance.range();
                let front_distance_range = front_distance.range();
                let right_distance_range = right_distance.range();

                let (left_power, right_power, debug) = mouse.update(
                    &config,
                    now,
                    battery.raw(),
                    left_encoder_count,
                    right_encoder_count,
                    left_distance_range,
                    front_distance_range,
                    right_distance_range,
                );

                right_motor.change_power((right_power * 10000.0 / 6.0) as i32);
                left_motor.change_power((left_power * 10000.0 / 6.0) as i32);

                if debugging && uart.tx_len() == Ok(0) {
                    let mut msgs = Vec::new();
                    msgs.push(DebugMsg::Orientation(debug.orientation.clone()));
                    msgs.push(DebugMsg::Motion(debug.motion.clone()));

                    //if step_count % 2 == 0 {
                    msgs.push(DebugMsg::Path(debug.path.clone()));
                    //}

                    let packet = DebugPacket {
                        msgs,
                        battery: debug.battery,
                        time: debug.time,
                        count: packet_count,
                    };

                    if let Ok(bytes) = postcard::to_vec::<U1024, _>(&packet) {
                        uart.add_bytes(&bytes);
                        orange_led.toggle().ok();
                    }

                    packet_count += 1;
                }

                step_count += 1;
            } else {
                right_motor.change_power(0);
                left_motor.change_power(0);
            }

            if left_button.is_low().unwrap() {
                running = true;
            }

            if right_button.is_low().unwrap() {
                running = false;
            }

            if battery.is_dead() {
                red_led.set_high().ok();
            } else {
                red_led.set_low().ok();
            }

            last_time = now;
        }

        battery.update(now);
    }
}
