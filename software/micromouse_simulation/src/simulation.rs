use std::f32;

use serde::Deserialize;
use serde::Serialize;

use micromouse_logic::fast::{
    Orientation, Vector, DIRECTION_0, DIRECTION_3_PI_2, DIRECTION_PI_2,
};
use micromouse_logic::mouse::{DistanceReading, Mouse, MouseConfig, MouseDebug};
use micromouse_logic::slow::maze::{
    Maze, MazeConfig, MazeIndex, MazeProjectionResult, Wall,
};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SimulationDebug {
    pub mouse: MouseDebug,
    pub left_encoder: i32,
    pub right_encoder: i32,
    pub left_wheel_speed: f32,
    pub right_wheel_speed: f32,
    pub left_accel: f32,
    pub right_accel: f32,
    pub left_ground_speed: f32,
    pub right_ground_speed: f32,
    pub left_distance: Option<DistanceReading>,
    pub front_distance: Option<DistanceReading>,
    pub right_distance: Option<DistanceReading>,
    pub orientation: Orientation,
    pub config: SimulationConfig,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub mouse: MouseConfig,
    pub initial_orientation: Orientation,
    pub millis_per_step: u32,
    pub millis_per_sensor_update: u32,

    /// The max speed a wheel can accelerate by before slipping
    pub max_wheel_accel: f32,
    pub max_speed: f32,

    pub maze: Maze,
}

impl SimulationConfig {
    pub fn sec_per_step(&self) -> f32 {
        self.millis_per_step as f32 / 1000.0
    }
}

/// Find the closest closed wall
fn find_closed_wall(
    config: &MazeConfig,
    maze: &Maze,
    from: Orientation,
) -> Option<MazeProjectionResult> {
    config.wall_projection(from).find(|maze_projection_result| {
        if let MazeIndex::Wall(wall_index) = maze_projection_result.maze_index {
            maze.get_wall(wall_index).unwrap_or(&Wall::Closed) == &Wall::Closed
        } else {
            true
        }
    })
}

pub struct Simulation {
    mouse: Mouse,
    orientation: Orientation,
    last_left_ground_speed: f32,
    last_right_ground_speed: f32,
    left_encoder: i32,
    right_encoder: i32,
    time: u32,
    last_sensor_update: u32,
}

impl Simulation {
    pub fn new(config: &SimulationConfig) -> Simulation {
        Simulation {
            mouse: Mouse::new(&config.mouse, config.initial_orientation, 0, 0, 0),
            orientation: config.initial_orientation,
            left_encoder: 0,
            right_encoder: 0,
            last_left_ground_speed: 0.0,
            last_right_ground_speed: 0.0,
            time: 0,
            last_sensor_update: 0,
        }
    }

    pub fn default_config() -> SimulationConfig {
        SimulationConfig::default()
    }

    pub fn update(&mut self, config: &SimulationConfig) -> SimulationDebug {
        let mech = config.mouse.mechanical;

        let (front_distance, left_distance, right_distance) =
            if self.time - self.last_sensor_update >= config.millis_per_sensor_update {
                // Figure out what the sensors should read
                let front_result = find_closed_wall(
                    &config.mouse.maze,
                    &config.maze,
                    self.orientation.offset(Orientation {
                        position: Vector {
                            x: config.mouse.mechanical.front_sensor_offset_x,
                            y: 0.0,
                        },

                        direction: DIRECTION_0,
                    }),
                );
                let front_distance = front_result
                    .filter(|result| result.distance < mech.front_sensor_limit as f32)
                    .map_or(DistanceReading::OutOfRange, |result| {
                        DistanceReading::InRange(result.distance)
                    });

                let left_result = find_closed_wall(
                    &config.mouse.maze,
                    &config.maze,
                    self.orientation.offset(Orientation {
                        position: Vector {
                            x: config.mouse.mechanical.left_sensor_offset_x,
                            y: config.mouse.mechanical.left_sensor_offset_y,
                        },
                        direction: DIRECTION_PI_2,
                    }),
                );
                let left_distance = left_result
                    .filter(|result| result.distance < mech.left_sensor_limit)
                    .map_or(DistanceReading::OutOfRange, |result| {
                        DistanceReading::InRange(result.distance)
                    });

                let right_result = find_closed_wall(
                    &config.mouse.maze,
                    &config.maze,
                    self.orientation.offset(Orientation {
                        position: Vector {
                            x: config.mouse.mechanical.right_sensor_offset_x,
                            y: -config.mouse.mechanical.right_sensor_offset_y,
                        },
                        direction: DIRECTION_3_PI_2,
                    }),
                );
                let right_distance = right_result
                    .filter(|result| result.distance < mech.right_sensor_limit as f32)
                    .map_or(DistanceReading::OutOfRange, |result| {
                        DistanceReading::InRange(result.distance)
                    });

                (
                    Some(front_distance),
                    Some(left_distance),
                    Some(right_distance),
                )
            } else {
                (None, None, None)
            };

        // Update the mouse for the current time
        let (raw_left_power, raw_right_power, mouse_debug) = self.mouse.update(
            &config.mouse,
            self.time,
            0,
            self.left_encoder,
            self.right_encoder,
            left_distance,
            front_distance,
            right_distance,
        );

        // Make sure the wheel powers are in range -1.0 to 1.0

        let left_power = if raw_left_power > 10000 {
            10000
        } else if raw_left_power < -10000 {
            -10000
        } else {
            raw_left_power
        };

        let right_power = if raw_right_power > 10000 {
            10000
        } else if raw_right_power < -10000 {
            -10000
        } else {
            raw_right_power
        };

        // Update the state for the next run
        let left_wheel_speed = left_power as f32 / 10000.0 * config.max_speed;
        let right_wheel_speed = right_power as f32 / 10000.0 * config.max_speed;

        let delta_left_wheel = config
            .mouse
            .mechanical
            .mm_to_ticks(left_wheel_speed * (config.millis_per_step as f32))
            as i32;

        let delta_right_wheel = config
            .mouse
            .mechanical
            .mm_to_ticks(right_wheel_speed * (config.millis_per_step as f32))
            as i32;

        let left_accel = (left_wheel_speed - self.last_left_ground_speed)
            / config.millis_per_step as f32;
        let right_accel = (right_wheel_speed - self.last_right_ground_speed)
            / config.millis_per_step as f32;

        let left_ground_speed = if left_accel > config.max_wheel_accel {
            self.last_left_ground_speed + config.max_wheel_accel
        } else {
            left_wheel_speed
        };

        let right_ground_speed = if right_accel > config.max_wheel_accel {
            self.last_right_ground_speed + config.max_wheel_accel
        } else {
            right_wheel_speed
        };

        let delta_left_ground = config
            .mouse
            .mechanical
            .mm_to_ticks(left_ground_speed * (config.millis_per_step as f32))
            as i32;

        let delta_right_ground = config
            .mouse
            .mechanical
            .mm_to_ticks(right_ground_speed * (config.millis_per_step as f32))
            as i32;

        // Collect debug info from this run
        let debug = SimulationDebug {
            mouse: mouse_debug,
            left_encoder: self.left_encoder,
            right_encoder: self.right_encoder,
            left_wheel_speed,
            right_wheel_speed,
            left_accel,
            right_accel,
            left_ground_speed,
            right_ground_speed,
            left_distance,
            front_distance,
            right_distance,
            orientation: self.orientation,
            config: config.clone(),
        };

        // Update for next run
        self.time += config.millis_per_step;
        self.left_encoder += delta_left_wheel;
        self.right_encoder += delta_right_wheel;
        self.last_left_ground_speed = left_ground_speed;
        self.last_right_ground_speed = right_ground_speed;
        self.orientation = self.orientation.update_from_encoders(
            &config.mouse.mechanical,
            delta_left_ground,
            delta_right_ground,
        );

        debug
    }
}
