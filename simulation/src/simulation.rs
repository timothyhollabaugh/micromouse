use std::f32;

use mouse::config::MouseConfig;
use mouse::map::Orientation;
use mouse::map::Vector;
use mouse::mouse::Mouse;
use mouse::mouse::MouseDebug;
use mouse::path::PathDebug;

#[derive(Debug)]
pub struct SimulationDebug<'a> {
    pub mouse_debug: MouseDebug<'a>,
    left_encoder: i32,
    right_encoder: i32,
}

pub struct SimulationConfig {
    pub mouse: MouseConfig,
    pub max_speed: f32,
    pub initial_orientation: Orientation,
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

    pub fn update(
        &mut self,
        config: &SimulationConfig,
        time: u32,
    ) -> (Orientation, &[Orientation], SimulationDebug) {
        let delta_time = time - self.time;

        let (left_power, right_power, mouse_debug) =
            self.mouse
                .update(&config.mouse, time, self.left_encoder, self.right_encoder);

        let left_speed = left_power * config.max_speed;
        let right_speed = right_power * config.max_speed;

        let delta_left = left_speed * delta_time as f32;
        let delta_right = right_speed * delta_time as f32;

        self.left_encoder += delta_left as i32;
        self.right_encoder += delta_right as i32;

        let delta_linear = config
            .mouse
            .mechanical
            .ticks_to_mm((delta_right + delta_left) as f32 / 2.0);

        let delta_angular = config
            .mouse
            .mechanical
            .ticks_to_rads((delta_right - delta_left) as f32 / 2.0);

        let mid_dir = self.orientation.direction + delta_angular / 2.0;

        self.past_orientations.push(self.orientation);

        self.orientation = Orientation {
            position: Vector {
                x: self.orientation.position.x + delta_linear * f32::cos(mid_dir),
                y: self.orientation.position.y + delta_linear * f32::sin(mid_dir),
            },

            direction: self.orientation.direction + delta_angular,
        };

        let debug = SimulationDebug {
            mouse_debug,
            left_encoder: self.left_encoder,
            right_encoder: self.right_encoder,
        };

        (self.orientation, self.past_orientations.as_ref(), debug)
    }
}
