use std::f32;

use std::io::BufRead;

use libm::F32Ext;

use serialport;
use serialport::SerialPort;

use mouse::config::MouseConfig;
use mouse::map::Direction;
use mouse::map::Orientation;
use mouse::map::Vector;
use mouse::mouse::Mouse;
use mouse::mouse::MouseDebug;
use mouse::path::PathDebug;

#[derive(Debug)]
pub struct SimulationDebug<'a> {
    pub mouse_debug: MouseDebug<'a>,
    pub left_encoder: i32,
    pub right_encoder: i32,
    pub orientation: Orientation,
    pub time: u32,
}

pub struct SimulationConfig {
    pub mouse: MouseConfig,
    pub max_speed: f32,
    pub initial_orientation: Orientation,
    pub millis_per_step: u32,
}

pub struct RemoteMouse<R: BufRead> {
    reader: R,
    orientation: Orientation,
}

impl<R: BufRead> RemoteMouse<R> {
    pub fn new(config: &SimulationConfig, reader: R) -> RemoteMouse<R> {
        RemoteMouse {
            reader,
            orientation: config.initial_orientation,
        }
    }

    pub fn update(&mut self, config: &SimulationConfig) -> SimulationDebug {
        let mut line = String::new();

        self.reader.read_line(&mut line);

        let parts = line
            .split(',')
            .map(|s| s.trim())
            .map(|s| s.parse::<f32>())
            .collect::<Vec<_>>();

        if let Some(&x) = parts.get(0).and_then(|r| r.as_ref().ok()) {
            self.orientation.position.x = x;
        }

        if let Some(&y) = parts.get(1).and_then(|r| r.as_ref().ok()) {
            self.orientation.position.y = y;
        }

        if let Some(&d) = parts.get(2).and_then(|r| r.as_ref().ok()) {
            self.orientation.direction = Direction::from(d);
        }

        SimulationDebug {
            mouse_debug: MouseDebug {
                orientation: self.orientation,
                path_debug: PathDebug {
                    path: None,
                    distance_from: None,
                    distance_along: None,
                    centered_direction: None,
                    tangent_direction: None,
                    target_direction: None,
                },
            },
            orientation: self.orientation,
            left_encoder: 0,
            right_encoder: 0,
            time: 0,
        }
    }
}

pub struct Simulation {
    mouse: Mouse,
    orientation: Orientation,
    past_orientations: Vec<Orientation>,
    left_encoder: i32,
    right_encoder: i32,
    time: u32,
}

impl Simulation {
    pub fn new(config: &SimulationConfig, time: u32) -> Simulation {
        Simulation {
            mouse: Mouse::new(&config.mouse, config.initial_orientation, 0, 0, 0),
            orientation: config.initial_orientation,
            past_orientations: Vec::new(),
            left_encoder: 0,
            right_encoder: 0,
            time,
        }
    }

    pub fn update(&mut self, config: &SimulationConfig) -> SimulationDebug {
        // Update the mouse for the current time
        let (left_power, right_power, mouse_debug) = self.mouse.update(
            &config.mouse,
            self.time,
            self.left_encoder,
            self.right_encoder,
        );

        // Collect debug info from this run
        let debug = SimulationDebug {
            mouse_debug,
            left_encoder: self.left_encoder,
            right_encoder: self.right_encoder,
            orientation: self.orientation,
            time: self.time,
        };

        // Update the state for the next run
        let left_speed = left_power * config.max_speed;
        let right_speed = right_power * config.max_speed;

        let delta_left = config
            .mouse
            .mechanical
            .mm_to_ticks(left_speed * (config.millis_per_step as f32 / 1000.0))
            as i32;

        let delta_right = config
            .mouse
            .mechanical
            .mm_to_ticks(right_speed * (config.millis_per_step as f32 / 1000.0))
            as i32;

        self.left_encoder += delta_left;
        self.right_encoder += delta_right;
        self.time += config.millis_per_step;

        self.orientation
            .update_from_encoders(&config.mouse.mechanical, delta_left, delta_right);

        debug
    }
}
