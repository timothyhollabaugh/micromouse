pub mod remote;
pub mod simulation;

use std::panic;

use wasm_bindgen::prelude::*;

use console_error_panic_hook;

use simulation::Simulation;
use simulation::SimulationConfig;

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
        let config: SimulationConfig = config.into_serde().expect("Could not parse config");
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
}
