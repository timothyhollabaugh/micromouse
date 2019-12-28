use std::cmp::min;
use std::f32;
use std::io::Read;

use serde::Deserialize;
use serde::Serialize;

use mouse::config::MouseConfig;
use mouse::map::Direction;
use mouse::map::MapDebug;
use mouse::map::Orientation;
use mouse::maze::Edge;
use mouse::maze::Maze;
use mouse::motion::MotionDebug;
use mouse::mouse::Mouse;
use mouse::mouse::MouseDebug;
use mouse::path::PathDebug;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SimulationDebug {
    pub mouse_debug: MouseDebug,
    pub left_encoder: i32,
    pub right_encoder: i32,
    pub orientation: Orientation,
    pub time: u32,
    pub config: SimulationConfig,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub mouse: MouseConfig,
    pub max_speed: f32,
    pub initial_orientation: Orientation,
    pub millis_per_step: u32,

    /// The max speed a wheel can accelerate by before slipping
    pub max_wheel_accel: f32,
}

impl SimulationConfig {
    pub fn sec_per_step(&self) -> f32 {
        self.millis_per_step as f32 / 1000.0
    }
}

pub struct RemoteMouse<R: Read> {
    reader: R,
    buf: String,
    debug: SimulationDebug,
}

impl<R: Read> RemoteMouse<R> {
    pub fn new(config: &SimulationConfig, reader: R) -> RemoteMouse<R> {
        RemoteMouse {
            reader,
            buf: String::new(),
            debug: SimulationDebug::default(),
        }
    }

    pub fn update(&mut self, _config: &SimulationConfig) -> SimulationDebug {
        self.reader.read_to_string(&mut self.buf).ok();

        if let Some(index) = self.buf.find('\n') {
            let line: String = self.buf.drain(0..(index + 1)).collect();

            eprintln!("line: {}", line);

            let mut parts = line.split(',').map(|s| s.trim());

            if let Some(Ok(centered_direction)) = parts.next().map(|p| p.parse()) {
                self.debug.mouse_debug.path.centered_direction = Some(centered_direction);
            }

            if let Some(Ok(target_direction)) = parts.next().map(|p| p.parse::<f32>()) {
                self.debug.mouse_debug.path.target_direction =
                    Some(Direction::from(target_direction));
            }

            /*
            if let Some(Ok(left_encoder)) = parts.next().map(|p| p.parse()) {
                self.left_encoder = left_encoder;
            }

            if let Some(Ok(right_encoder)) = parts.next().map(|p| p.parse()) {
                self.right_encoder = right_encoder;
            }

            if let Some(Ok(x)) = parts.next().map(|p| p.parse()) {
                self.orientation.position.x = x;
            }

            if let Some(Ok(y)) = parts.next().map(|p| p.parse()) {
                self.orientation.position.y = y;
            }

            if let Some(Ok(d)) = parts.next().map(|p| p.parse::<f32>()) {
                self.orientation.direction = Direction::from(d)
            }
            */
        }
        self.debug.clone()
    }
}

pub struct Simulation {
    mouse: Mouse,
    orientation: Orientation,
    past_orientations: Vec<Orientation>,
    last_left_wheel_speed: f32,
    last_right_wheel_speed: f32,
    left_encoder: i32,
    right_encoder: i32,
    time: u32,
}

impl Simulation {
    pub fn new(config: &SimulationConfig) -> Simulation {
        Simulation {
            mouse: Mouse::new(&config.mouse, config.initial_orientation, 0, 0, 0),
            orientation: config.initial_orientation,
            past_orientations: Vec::new(),
            left_encoder: 0,
            right_encoder: 0,
            last_left_wheel_speed: 0.0,
            last_right_wheel_speed: 0.0,
            time: 0,
        }
    }

    pub fn update(&mut self, config: &SimulationConfig) -> SimulationDebug {
        // Update the mouse for the current time
        let (left_power, right_power, mouse_debug) = self.mouse.update(
            &config.mouse,
            self.time,
            self.left_encoder,
            self.right_encoder,
            255,
            255,
            255,
        );

        //let (left_power, right_power, mouse_debug): (f32, f32, MouseDebug) = Default::default();

        // Update the state for the next run
        let left_wheel_speed = left_power * config.max_speed;
        let right_wheel_speed = right_power * config.max_speed;

        let delta_left_wheel = config
            .mouse
            .mechanical
            .mm_to_ticks(left_wheel_speed * (config.millis_per_step as f32 / 1000.0))
            as i32;

        let delta_right_wheel = config
            .mouse
            .mechanical
            .mm_to_ticks(right_wheel_speed * (config.millis_per_step as f32 / 1000.0))
            as i32;

        self.left_encoder += delta_left_wheel;
        self.right_encoder += delta_right_wheel;
        self.time += config.millis_per_step;

        let left_accel = (left_wheel_speed - self.last_left_wheel_speed) / config.sec_per_step();
        let right_accel = (right_wheel_speed - self.last_right_wheel_speed) / config.sec_per_step();

        let left_ground_speed = if left_accel > config.max_wheel_accel {
            self.last_left_wheel_speed + config.max_wheel_accel * config.sec_per_step()
        } else {
            left_wheel_speed
        };

        let right_ground_speed = if right_accel > config.max_wheel_accel {
            self.last_right_wheel_speed + config.max_wheel_accel * config.sec_per_step()
        } else {
            right_wheel_speed
        };

        let delta_left_ground = config
            .mouse
            .mechanical
            .mm_to_ticks(left_ground_speed * config.sec_per_step())
            as i32;

        let delta_right_ground = config
            .mouse
            .mechanical
            .mm_to_ticks(right_ground_speed * config.sec_per_step())
            as i32;

        self.orientation.update_from_encoders(
            &config.mouse.mechanical,
            delta_left_ground,
            delta_right_ground,
        );

        // Collect debug info from this run
        let debug = SimulationDebug {
            mouse_debug,
            left_encoder: self.left_encoder,
            right_encoder: self.right_encoder,
            orientation: self.orientation,
            time: self.time,
            config: config.clone(),
        };

        debug
    }
}
