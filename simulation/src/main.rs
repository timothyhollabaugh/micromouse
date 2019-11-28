extern crate piston_window;

mod gui;
mod simulation;

use mouse::config::MouseConfig;
use mouse::config::MOUSE_2019_MECH;
use mouse::config::MOUSE_MAZE_MAP;
use mouse::config::MOUSE_SIM_PATH;
use mouse::map::{Direction, Orientation, Vector};

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
            max_wheel_accel: 60000.0,
        },

        pixels_per_mm: 0.25,
        time_scale: 1.0,
        simulated_mouse_color: [0.0, 1.0, 0.0, 1.0],
        real_mouse_color: [1.0, 0.0, 0.0, 1.0],
        path_color: [0.0, 0.0, 1.0, 1.0],
        wall_open_color: [1.0, 1.0, 1.0, 1.0],
        wall_closed_color: [0.5, 0.5, 0.5, 1.0],
        wall_unknown_color: [0.9, 0.9, 0.9, 1.0],
        wall_err_color: [1.0, 0.0, 0.0, 1.0],
        wall_front_border_color: [1.0, 0.0, 1.0, 1.0],
        wall_left_border_color: [1.0, 1.0, 0.0, 1.0],
        wall_right_border_color: [0.0, 1.0, 1.0, 1.0],
        post_color: [0.0, 0.0, 0.0, 1.0],
    };

    gui::run(config);
}
