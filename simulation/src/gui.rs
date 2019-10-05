use std::time::Instant;

use piston_window::clear;
use piston_window::rectangle;
use piston_window::EventLoop;
use piston_window::PistonWindow;
use piston_window::RenderEvent;
use piston_window::Transformed;
use piston_window::UpdateEvent;
use piston_window::WindowSettings;

use mouse::maze::HEIGHT;
use mouse::maze::WIDTH;

use crate::simulation::Simulation;
use crate::simulation::SimulationConfig;

pub struct GuiConfig {
    pub simulation: SimulationConfig,
    pub pixels_per_mm: f32,
    pub updates_per_second: f32,
    pub frames_per_second: f32,
}

impl GuiConfig {
    pub fn pixels_per_cell(&self) -> f32 {
        self.simulation.mouse.map.cell_width * self.pixels_per_mm
    }

    pub fn pixels_per_wall(&self) -> f32 {
        self.simulation.mouse.map.wall_width * self.pixels_per_mm
    }
}

pub fn run(config: GuiConfig) {
    let maze_size = (
        (WIDTH as f32 * config.pixels_per_cell()) as u32,
        (HEIGHT as f32 * config.pixels_per_cell()) as u32,
    );

    let mut window: PistonWindow = WindowSettings::new("Micromouse Simulation", maze_size)
        .exit_on_esc(true)
        .build()
        .unwrap();

    window.set_ups(config.updates_per_second as u64);
    window.set_max_fps(config.frames_per_second as u64);

    let mut simulation = Simulation::new(&config.simulation, 0);
    let mut orientation = config.simulation.initial_orientation;

    let start_time = Instant::now();

    while let Some(event) = window.next() {
        if let Some(_u) = event.update_args() {
            let time = std::time::Instant::now()
                .duration_since(start_time)
                .as_millis();
            orientation = simulation.update(&config.simulation, time as u32)
        }

        if let Some(_r) = event.render_args() {
            window.draw_2d(&event, |context, graphics| {
                clear([1.0; 4], graphics);

                let transform = context
                    .transform
                    .trans(
                        (orientation.position.x * config.pixels_per_mm) as f64,
                        maze_size.1 as f64 - (orientation.position.y * config.pixels_per_mm) as f64,
                    )
                    .rot_rad(-orientation.direction as f64);

                rectangle(
                    [0.0, 1.0, 0.0, 1.0],
                    [
                        (-config.simulation.mouse.mechanical.width * config.pixels_per_mm / 2.0)
                            as f64,
                        (-config.simulation.mouse.mechanical.length * config.pixels_per_mm / 2.0)
                            as f64,
                        (config.simulation.mouse.mechanical.width * config.pixels_per_mm) as f64,
                        (config.simulation.mouse.mechanical.length * config.pixels_per_mm) as f64,
                    ],
                    transform,
                    graphics,
                );
            });
        }
    }
}
