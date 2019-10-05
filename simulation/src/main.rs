extern crate piston_window;

mod gui;
mod simulation;

use mouse::config::MechanicalConfig;
use mouse::config::MouseConfig;
use mouse::map::{MapConfig, Orientation, Vector};
use mouse::path::PathConfig;

use simulation::SimulationConfig;

use gui::GuiConfig;

fn main() {
    let config = GuiConfig {
        simulation: SimulationConfig {
            mouse: MouseConfig {
                mechanical: MechanicalConfig {
                    wheel_diameter: 32.0,
                    gearbox_ratio: 75.0,
                    ticks_per_rev: 12.0,
                    wheelbase: 72.0,
                    width: 64.0,
                    length: 88.0,
                    front_offset: 48.0,
                },

                path: PathConfig {
                    p: 1.0,
                    i: 0.0,
                    d: 0.0,
                },

                map: MapConfig {
                    cell_width: 180.0,
                    wall_width: 20.0,
                },
            },

            max_speed: 500.0,

            initial_orientation: Orientation {
                position: Vector { x: 90.0, y: 90.0 },
                direction: 0.0,
            },
        },

        pixels_per_mm: 0.25,
        updates_per_second: 20.0,
        frames_per_second: 20.0,
    };

    gui::run(config);
}
