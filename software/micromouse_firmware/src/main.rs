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
pub mod system_test;
pub mod time;
pub mod uart;
pub mod vl6180x;

use cortex_m_rt::entry;
use stm32f4xx_hal as stm32f4;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::stm32 as stm32f405;

use heapless::Vec;

use postcard;

use embedded_hal::blocking::i2c;
use embedded_hal::digital::v2::{InputPin, OutputPin, ToggleableOutputPin};

use typenum::consts::U2048;

use crate::battery::Battery;
use crate::time::Time;

use crate::uart::Uart;

use crate::motors::{Encoder, Motor};

#[allow(unused_imports)]
use micromouse_logic::config::{mouse_2019, mouse_2020};

use micromouse_logic::comms::{DebugMsg, DebugPacket};
use micromouse_logic::fast::{Orientation, Vector, DIRECTION_PI_2};
use micromouse_logic::mouse::Mouse;

use crate::motors::left::{LeftEncoder, LeftMotor};
use crate::motors::right::{RightEncoder, RightMotor};
use crate::vl6180x::VL6180x;
use micromouse_logic::fast::motion_control::MotionHandlerDebug;

// Setup the master clock out
pub fn mco2_setup(rcc: &stm32f405::RCC, gpioc: &stm32f405::GPIOC) {
    rcc.ahb1enr.write(|w| w.gpiocen().set_bit());
    rcc.cfgr.modify(|_, w| w.mco2().sysclk());
    gpioc.moder.write(|w| w.moder9().alternate());
    gpioc.afrh.write(|w| w.afrh9().af0());
}

pub fn do_mouse<RL, GL, BL, OL, LB, RB, I2C1, I2C2, I2C3>(
    mut time: Time,
    mut battery: Battery,
    mut red_led: RL,
    mut green_led: GL,
    mut blue_led: BL,
    mut orange_led: OL,
    left_button: LB,
    right_button: RB,
    mut left_motor: LeftMotor,
    mut right_motor: RightMotor,
    left_encoder: LeftEncoder,
    right_encoder: RightEncoder,
    mut front_distance: VL6180x<I2C1>,
    mut left_distance: VL6180x<I2C2>,
    mut right_distance: VL6180x<I2C3>,
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
    let config = mouse_2020::MOUSE;

    let initial_orientation = Orientation {
        position: Vector {
            //x: 1260.0,
            //y: 1170.0,
            //x: 1250.0,
            //y: 1350.0,
            x: 90.0,
            y: 90.0,
        },
        direction: DIRECTION_PI_2,
    };

    let mut last_time: u32 = time.now();

    let mut mouse: Option<Mouse> = None;

    let mut debugging = false;

    let mut start_time = None;
    let mut last_packet_time = last_time;
    let mut packet_count = 0;

    let mut sensor_updating = 0;

    loop {
        let now: u32 = time.now();

        match sensor_updating {
            0 => {
                front_distance.update();
                sensor_updating += 1;
            }
            1 => {
                right_distance.update();
                sensor_updating += 1;
            }
            _ => {
                left_distance.update();
                sensor_updating = 0;
            }
        }

        if let Ok(byte) = uart.read_byte() {
            //blue_led.set_high().ok();
            match byte {
                0 => {}
                1 => debugging = false,
                2 => debugging = true,
                3 => {
                    mouse = None;
                    start_time = None;
                }
                4 => {
                    start_time = Some(now);
                }
                _ => {}
            }
        } else {
            //blue_led.set_low().ok();
        }

        if now - last_time >= 10 {
            green_led.toggle().ok();

            let debug = if let Some(mouse) = mouse.as_mut() {
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

                right_motor.change_power((right_power) as i32);
                left_motor.change_power((left_power) as i32);

                match debug.motion_control.handler {
                    Some(MotionHandlerDebug::Turn(_)) => blue_led.set_high().ok(),
                    _ => blue_led.set_low().ok(),
                };

                if let Some(_) = debug.slow {
                    orange_led.set_high().ok();
                } else {
                    orange_led.set_low().ok();
                }

                Some(debug)
            } else {
                right_motor.change_power(0);
                left_motor.change_power(0);
                None
            };

            if let Some(start_time) = start_time {
                if now - start_time > 0 && debugging && uart.tx_len() == Ok(0) {
                    let mut msgs = Vec::new();

                    if let Some(debug) = debug {
                        msgs.push(DebugMsg::Orientation(debug.orientation)).ok();
                        msgs.push(DebugMsg::Hardware(debug.hardware)).ok();
                        msgs.push(DebugMsg::Slow(debug.slow)).ok();
                        msgs.push(DebugMsg::Localize(debug.localize)).ok();
                        //msgs.push(DebugMsg::MotionQueue(debug.motion_queue)).ok();
                        //msgs.push(DebugMsg::MotorControl(
                        //debug.motion_control.motor_control,
                        //))
                        //.ok();
                        //msgs.push(DebugMsg::MotionHandler(debug.motion_control.handler))
                        //.ok();
                    }

                    let packet = DebugPacket {
                        msgs,
                        battery: battery.raw(),
                        time: now,
                        delta_time_sys: now - last_time,
                        delta_time_msg: now - last_packet_time,
                        count: packet_count,
                    };

                    if let Ok(bytes) = postcard::to_vec::<U2048, _>(&packet) {
                        uart.add_bytes(&bytes).ok();
                        //orange_led.set_high().ok();
                    }

                    packet_count += 1;
                    last_packet_time = now;
                } else {
                    //orange_led.set_low().ok();
                }

                if now - start_time > 1000 && mouse.is_none() {
                    mouse = Some(Mouse::new(
                        &config,
                        initial_orientation,
                        last_time,
                        left_encoder.count(),
                        right_encoder.count(),
                    ))
                }
            }

            if let Ok(true) = left_button.is_low() {
                start_time = Some(now);
            }

            if let Ok(true) = right_button.is_low() {
                mouse = None;
                start_time = None;
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

#[entry]
fn main() -> ! {
    let p = stm32f4::stm32::Peripherals::take().unwrap();
    let _cp = stm32f405::CorePeripherals::take().unwrap();

    // Enable pll on mco2
    //p.RCC.ahb1enr.write(|w| w.gpiocen().set_bit());
    //p.RCC.cfgr.modify(|_, w| w.mco2().sysclk().mco2pre().div5());
    //p.GPIOC.moder.write(|w| w.moder9().alternate());
    //p.GPIOC.afrh.write(|w| w.afrh9().af0());

    // Init non-hal things
    let mut time = Time::setup(&p.RCC, p.TIM1);

    while time.now() < 10000 {}

    let battery = Battery::setup(&p.RCC, &p.GPIOB, p.ADC1);

    let mut uart = Uart::setup(&p.RCC, p.USART1, &p.GPIOA);

    let left_motor = LeftMotor::setup(&p.RCC, p.TIM3, &p.GPIOA);
    let left_encoder = LeftEncoder::setup(&p.RCC, &p.GPIOA, &p.GPIOB, p.TIM2);

    let right_motor = RightMotor::setup(&p.RCC, p.TIM4, &p.GPIOB);
    let right_encoder = RightEncoder::setup(&p.RCC, &p.GPIOA, p.TIM5);

    // Init the hal things
    let rcc = p.RCC.constrain();
    let clocks = rcc
        .cfgr
        .hclk(168.mhz())
        .sysclk(168.mhz())
        .pclk2(84.mhz())
        .pclk1(42.mhz())
        .freeze();

    let gpioa = p.GPIOA.split();
    let gpiob = p.GPIOB.split();
    let gpioc = p.GPIOC.split();

    let mut red_led = gpiob.pb12.into_push_pull_output();
    let mut green_led = gpiob.pb13.into_push_pull_output();
    let mut blue_led = gpiob.pb14.into_push_pull_output();
    let mut orange_led = gpiob.pb15.into_push_pull_output();

    red_led.set_high().ok();
    green_led.set_high().ok();
    orange_led.set_high().ok();
    blue_led.set_high().ok();

    let left_button = gpioc.pc10.into_pull_up_input();
    let _middle_button = gpioc.pc11.into_pull_up_input();
    let right_button = gpioc.pc12.into_pull_up_input();

    time.delay(6000);

    red_led.set_low().ok();
    green_led.set_low().ok();
    orange_led.set_high().ok();
    blue_led.set_low().ok();

    uart.add_bytes(b"Initializing\n").ok();
    let mut left_distance = {
        let scl = gpiob.pb10.into_open_drain_output().into_alternate_af4();
        let sda = gpiob.pb11.into_open_drain_output().into_alternate_af4();

        let mut gpio0 = gpioc.pc2.into_open_drain_output();
        gpio0.set_high().ok();

        let mut gpio1 = gpioc.pc3.into_open_drain_output();
        gpio1.set_high().ok();

        let i2c = stm32f4::i2c::I2c::i2c2(p.I2C2, (scl, sda), 100.khz(), clocks);

        time.delay(100);

        let mut distance = vl6180x::VL6180x::new(i2c, 0x29);
        distance.init_private_registers();
        distance.init_default();
        distance
    };

    orange_led.set_high().ok();
    blue_led.set_high().ok();

    let mut front_distance = {
        let scl = gpiob.pb8.into_open_drain_output().into_alternate_af4();
        let sda = gpiob.pb9.into_open_drain_output().into_alternate_af4();

        let mut gpio0 = gpioc.pc0.into_open_drain_output();
        gpio0.set_high().ok();

        let mut gpio1 = gpioc.pc1.into_open_drain_output();
        gpio1.set_high().ok();

        let i2c = stm32f4::i2c::I2c::i2c1(p.I2C1, (scl, sda), 100.khz(), clocks);

        time.delay(100);

        let mut distance = vl6180x::VL6180x::new(i2c, 0x29);
        distance.init_private_registers();
        distance.init_default();
        distance
    };

    orange_led.set_low().ok();
    blue_led.set_high().ok();

    let mut right_distance = {
        let scl = gpioa.pa8.into_open_drain_output().into_alternate_af4();
        let sda = gpioc.pc9.into_open_drain_output().into_alternate_af4();

        let mut gpio0 = gpioc.pc4.into_open_drain_output();
        gpio0.set_high().ok();

        let mut gpio1 = gpioc.pc5.into_open_drain_output();
        gpio1.set_high().ok();

        let i2c = stm32f4::i2c::I2c::i2c3(p.I2C3, (scl, sda), 100.khz(), clocks);

        time.delay(100);

        let mut distance = vl6180x::VL6180x::new(i2c, 0x29);
        distance.init_private_registers();
        distance.init_default();
        distance
    };

    blue_led.set_low().ok();
    orange_led.set_low().ok();

    front_distance.start_ranging();
    left_distance.start_ranging();
    right_distance.start_ranging();

    uart.add_bytes(b"\n\nstart").ok();

    do_mouse(
    //do_sensors(
    //do_echo(
    //system_test::do_system_test(
        time,
        battery,
        red_led,
        green_led,
        blue_led,
        orange_led,
        left_button,
        right_button,
        left_motor,
        right_motor,
        left_encoder,
        right_encoder,
        front_distance,
        left_distance,
        right_distance,
        uart,
    );
}
