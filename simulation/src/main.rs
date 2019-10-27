extern crate piston_window;

mod gui;
mod simulation;

use mouse::config::MechanicalConfig;
use mouse::config::MouseConfig;
use mouse::config::MOUSE_2019_MECH;
use mouse::config::MOUSE_MAZE_MAP;
use mouse::config::MOUSE_SIM_PATH;
use mouse::map::{Direction, MapConfig, Orientation, Vector};
use mouse::path::PathConfig;

use simulation::SimulationConfig;

use gui::GuiConfig;

fn main() {
    let config = GuiConfig {
        simulation: SimulationConfig {
            mouse: MouseConfig {
                mechanical: MOUSE_2019_MECH,
                path: MOUSE_SIM_PATH,
                map: MOUSE_MAZE_MAP,
            },

            max_speed: 500.0,

            initial_orientation: Orientation {
                position: Vector {
                    x: 1000.0,
                    y: 1000.0,
                },
                direction: Direction::from(0.0),
            },

            millis_per_step: 10,
        },

        pixels_per_mm: 0.25,
        time_scale: 1.0,
    };

    gui::run(config);
}
