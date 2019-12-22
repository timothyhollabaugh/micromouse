pub mod simulation;

use std::panic;

use wasm_bindgen::prelude::*;

use console_error_panic_hook;

use simulation::Simulation;
use simulation::SimulationConfig;
use simulation::SimulationDebug;

/// A wrapper for an actual Simulation that handles javascript
/// type conversions
#[wasm_bindgen]
pub struct JsSimulation {
    simulation: Simulation,
}

#[wasm_bindgen]
impl JsSimulation {
    /// Create a new simulation
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> JsSimulation {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        let config: SimulationConfig = config.into_serde().expect("Could not parse config");
        JsSimulation {
            simulation: Simulation::new(&config),
        }
    }

    /// Update the simulation
    /// The config argument is a SimulationConfig,
    /// and the return is a SimulationDebug.
    pub fn update(&mut self, config: JsValue) -> JsValue {
        //let config: SimulationConfig = config.into_serde().unwrap();
        let config = SimulationConfig::default();
        let debug = self.simulation.update(&config);
        JsValue::from_serde(&debug).unwrap()
    }
}
