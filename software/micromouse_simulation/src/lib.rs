pub mod remote;
pub mod simulation;

use std::panic;

use wasm_bindgen::prelude::*;

use console_error_panic_hook;

#[allow(unused_imports)]
use micromouse_logic::config::*;

use simulation::Simulation;
use simulation::SimulationConfig;

use micromouse_logic::config::sim::MOUSE_2019;
use micromouse_logic::fast::{Orientation, Vector, DIRECTION_PI_2};
use micromouse_logic::slow::maze::Maze;
use remote::Remote;
use remote::RemoteConfig;

#[wasm_bindgen]
pub fn init_wasm() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

/// A wrapper for an actual Simulation that handles javascript
/// type conversions
#[wasm_bindgen]
pub struct JsSimulation {
    simulation: Simulation,
    config: SimulationConfig,
}

#[wasm_bindgen]
impl JsSimulation {
    /// Create a new simulation
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> JsSimulation {
        let config: SimulationConfig =
            config.into_serde().expect("Could not parse config");
        JsSimulation {
            simulation: Simulation::new(&config),
            config,
        }
    }

    /// Update the simulation
    /// The return is a SimulationDebug.
    pub fn update(&mut self) -> JsValue {
        let debug = self.simulation.update(&self.config);
        JsValue::from_serde(&debug).unwrap()
    }

    pub fn config(&mut self, config: JsValue) {
        self.config = config.into_serde().expect("Could not parse config");
    }

    pub fn default_config() -> JsValue {
        /*
        let mut horizontal_walls = [[Wall::Unknown; maze::HEIGHT - 1]; maze::WIDTH];
        let mut vertical_walls = [[Wall::Unknown; maze::HEIGHT]; maze::WIDTH - 1];

        horizontal_walls[6][8] = Wall::Closed;
        horizontal_walls[7][8] = Wall::Closed;
        horizontal_walls[8][8] = Wall::Closed;
        horizontal_walls[9][8] = Wall::Closed;

        horizontal_walls[6][7] = Wall::Open;
        horizontal_walls[7][7] = Wall::Closed;
        horizontal_walls[8][7] = Wall::Closed;
        horizontal_walls[9][7] = Wall::Open;

        horizontal_walls[6][6] = Wall::Open;
        horizontal_walls[7][6] = Wall::Closed;
        horizontal_walls[8][6] = Wall::Closed;
        horizontal_walls[9][6] = Wall::Open;

        horizontal_walls[6][5] = Wall::Closed;
        horizontal_walls[7][5] = Wall::Closed;
        horizontal_walls[8][5] = Wall::Closed;
        horizontal_walls[9][5] = Wall::Closed;

        vertical_walls[5][8] = Wall::Closed;
        vertical_walls[5][7] = Wall::Closed;
        vertical_walls[5][6] = Wall::Closed;

        vertical_walls[6][8] = Wall::Open;
        vertical_walls[6][7] = Wall::Closed;
        vertical_walls[6][6] = Wall::Open;

        vertical_walls[7][8] = Wall::Open;
        vertical_walls[7][7] = Wall::Open;
        vertical_walls[7][6] = Wall::Open;

        vertical_walls[8][8] = Wall::Open;
        vertical_walls[8][7] = Wall::Closed;
        vertical_walls[8][6] = Wall::Open;

        vertical_walls[9][8] = Wall::Closed;
        vertical_walls[9][7] = Wall::Closed;
        vertical_walls[9][6] = Wall::Closed;

        let maze = Maze::from_walls(horizontal_walls, vertical_walls);
        */
        let bytes = include_bytes!("../mazes/APEC2017.maz");
        let maze = Maze::from_file(*bytes);

        JsValue::from_serde(&SimulationConfig {
            mouse: MOUSE_2019,
            millis_per_step: 10,
            millis_per_sensor_update: 20,
            initial_orientation: Orientation {
                position: Vector {
                    x: 0.5 * 180.0,
                    y: 0.5 * 180.0,
                },
                direction: DIRECTION_PI_2,
            },
            max_wheel_accel: 1.0,
            max_speed: 1.0,
            maze,
        })
        .unwrap()
    }
}

#[wasm_bindgen]
pub struct JsRemote {
    remote: Remote,
}

#[wasm_bindgen]
impl JsRemote {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> JsRemote {
        let config: RemoteConfig = config.into_serde().expect("Could not parse config");
        JsRemote {
            remote: Remote::new(&config),
        }
    }

    pub fn update(&mut self, bytes: Vec<u8>) -> JsValue {
        let debugs = self.remote.update(&bytes);
        JsValue::from_serde(&debugs).unwrap()
    }

    pub fn default_config() -> JsValue {
        JsValue::from_serde(&RemoteConfig { mouse: MOUSE_2019 }).unwrap()
    }
}
