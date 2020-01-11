pub mod remote;
pub mod simulation;

use std::panic;

use wasm_bindgen::prelude::*;

use console_error_panic_hook;

#[allow(unused_imports)]
use micromouse_logic::config::*;

use simulation::Simulation;
use simulation::SimulationConfig;

use micromouse_logic::map::{Direction, Orientation, Vector};
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
        JsValue::from_serde(&SimulationConfig {
            mouse: MOUSE_SIM_2019,
            max_wheel_accel: 60000.0,
            millis_per_step: 10,
            initial_orientation: Orientation {
                position: Vector {
                    x: 90.0,
                    y: 6.0 * 180.0,
                },
                direction: Direction::from(0.0),
            },
            max_speed: 1000.0,
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
        let config: RemoteConfig =
            config.into_serde().expect("Could not parse config");
        JsRemote {
            remote: Remote::new(&config),
        }
    }

    pub fn update(&mut self, bytes: Vec<u8>) -> JsValue {
        let debugs = self.remote.update(&bytes);
        JsValue::from_serde(&debugs).unwrap()
    }

    pub fn default_config() -> JsValue {
        JsValue::from_serde(&RemoteConfig {
            mouse: MOUSE_SIM_2019,
        })
        .unwrap()
    }
}
