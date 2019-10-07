use std::time::Instant;

use piston_window::circle_arc;
use piston_window::clear;
use piston_window::line;
use piston_window::rectangle;
use piston_window::AdvancedWindow;
use piston_window::EventLoop;
use piston_window::PistonWindow;
use piston_window::RenderEvent;
use piston_window::Transformed;
use piston_window::UpdateEvent;
use piston_window::WindowSettings;

use mouse::maze::HEIGHT;
use mouse::maze::WIDTH;

use mouse::path::Segment;

use crate::simulation::Simulation;
use crate::simulation::SimulationConfig;
use mouse::map::Orientation;

pub struct GuiConfig {
    pub simulation: SimulationConfig,
    pub pixels_per_mm: f32,
    pub time_scale: f32,
}

impl GuiConfig {
    pub fn pixels_per_cell(&self) -> f32 {
        self.simulation.mouse.map.cell_width * self.pixels_per_mm
    }

    pub fn pixels_per_wall(&self) -> f32 {
        self.simulation.mouse.map.wall_width * self.pixels_per_mm
    }
}

fn orientation_transform<T: Transformed + Sized>(orientation: &Orientation, transform: T) -> T {
    transform
        .trans(orientation.position.x as f64, orientation.position.y as f64)
        .rot_rad(orientation.direction as f64)
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

    window.set_ups(
        (1000.0 / (config.simulation.millis_per_step as f64) * config.time_scale as f64) as u64,
    );

    window.set_max_fps(50);

    let mut simulation = Simulation::new(&config.simulation, 0);

    let mut debug = simulation.update(&config.simulation);

    while let Some(event) = window.next() {
        if let Some(u) = event.update_args() {
            debug = simulation.update(&config.simulation);
        }

        if let Some(r) = event.render_args() {
            window.draw_2d(&event, |context, graphics| {
                clear([1.0; 4], graphics);

                let transform = context
                    .transform
                    .scale(config.pixels_per_mm as f64, config.pixels_per_mm as f64);

                if let Some(path) = debug.mouse_debug.path_debug.path {
                    for segment in path {
                        match segment {
                            &Segment::Line(l1, l2) => line(
                                [0.0, 0.0, 1.0, 1.0],
                                2.0,
                                [l1.x as f64, l1.y as f64, l2.x as f64, l2.y as f64],
                                transform,
                                graphics,
                            ),
                            &Segment::Arc(s, c, t) => {
                                let v = s - c;
                                let r = v.magnitude();

                                let (t_start, t_end) = if t < 0.0 {
                                    let t_start = f32::atan2(v.y, v.x);
                                    let t_end = t_start - t;
                                    (t_start, t_end)
                                } else {
                                    let t_start = f32::atan2(v.y, v.x);
                                    let t_end = t_start - t;
                                    (t_end, t_start)
                                };

                                circle_arc(
                                    [0.0, 0.0, 1.0, 1.0],
                                    2.0,
                                    t_start as f64,
                                    t_end as f64,
                                    [
                                        (c.x - r) as f64,
                                        (c.y - r) as f64,
                                        (r * 2.0) as f64,
                                        (r * 2.0) as f64,
                                    ],
                                    transform,
                                    graphics,
                                )
                            }
                        }
                    }
                }
                rectangle(
                    [0.0, 1.0, 0.0, 1.0],
                    [
                        (-config.simulation.mouse.mechanical.length / 2.0) as f64,
                        (-config.simulation.mouse.mechanical.width / 2.0) as f64,
                        config.simulation.mouse.mechanical.length as f64,
                        config.simulation.mouse.mechanical.width as f64,
                    ],
                    orientation_transform(&debug.orientation, transform),
                    graphics,
                );
            });
        }
    }
}
