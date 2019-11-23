use std::time::Instant;
use std::time::Duration;

use std::thread;

use std::sync::mpsc;

use std::f32::consts::PI;

use std::io::BufReader;

use plotters::prelude::*;

use image::ImageBuffer;

use piston_window::circle_arc;
use piston_window::clear;
use piston_window::image as draw_image;
use piston_window::line;
use piston_window::rectangle;
use piston_window::rectangle::Border;
use piston_window::rectangle::Rectangle;
use piston_window::AdvancedWindow;
use piston_window::EventLoop;
use piston_window::PistonWindow;
use piston_window::RenderEvent;
use piston_window::Texture;
use piston_window::TextureSettings;
use piston_window::Transformed;
use piston_window::UpdateEvent;
use piston_window::WindowSettings;

use mouse::maze::Edge;
use mouse::maze::EdgeIndex;
use mouse::maze::HEIGHT;
use mouse::maze::WIDTH;

use mouse::map::Orientation;
use mouse::path::Segment;

use crate::simulation::RemoteMouse;
use crate::simulation::Simulation;
use crate::simulation::SimulationConfig;
use crate::simulation::SimulationDebug;

#[derive(Debug, Copy, Clone)]
pub struct GuiConfig {
    pub simulation: SimulationConfig,
    pub pixels_per_mm: f32,
    pub time_scale: f32,
    pub mouse_color: [f32; 4],
    pub path_color: [f32; 4],
    pub wall_open_color: [f32; 4],
    pub wall_closed_color: [f32; 4],
    pub wall_unknown_color: [f32; 4],
    pub wall_err_color: [f32; 4],
    pub wall_front_border_color: [f32; 4],
    pub post_color: [f32; 4],
}

impl GuiConfig {
    pub fn pixels_per_cell(&self) -> f32 {
        self.simulation.mouse.map.maze.cell_width * self.pixels_per_mm
    }

    pub fn pixels_per_wall(&self) -> f32 {
        self.simulation.mouse.map.maze.wall_width * self.pixels_per_mm
    }
}

fn orientation_transform<T: Transformed + Sized>(orientation: &Orientation, transform: T) -> T {
    transform
        .trans(orientation.position.x as f64, orientation.position.y as f64)
        .rot_rad(orientation.direction.into())
}

enum GuiCmd {
    Exit
}

pub fn run(config: GuiConfig) {

    let (debug_tx, debug_rx) = mpsc::channel();
    let (cmd_tx, cmd_rx) = mpsc::channel();

    let simulation_thread = thread::spawn(move || run_simulation(debug_tx, cmd_rx, &config));
    let gui_thread = thread::spawn(move || run_gui(debug_rx, cmd_tx, &config.clone()));

    simulation_thread.join();
    gui_thread.join();
}

fn run_simulation(debug_tx: mpsc::Sender<SimulationDebug>, cmd_rx: mpsc::Receiver<GuiCmd>, config: &GuiConfig) {
    let mut simulation = Simulation::new(&config.simulation, 0);

    //let serial = serialport::open("/dev/rfcomm0").unwrap();
    //let mut simulation = RemoteMouse::new(&config.simulation, serial);

    'main: loop {
        for cmd in cmd_rx.try_iter() {
            match cmd {
                Exit => break 'main,
            }
        }

        let debug = simulation.update(&config.simulation);

        debug_tx.send(debug);

        thread::sleep(Duration::from_millis((config.simulation.millis_per_step as f32 * config.time_scale) as u64));
    }
}

fn run_gui(debug_rx: mpsc::Receiver<SimulationDebug>, cmd_tx: mpsc::Sender<GuiCmd>, config: &GuiConfig) {
    let maze_size = (
        (WIDTH as f32 * config.pixels_per_cell()) as u32,
        (HEIGHT as f32 * config.pixels_per_cell()) as u32,
    );

    let mut window: PistonWindow = WindowSettings::new("Micromouse Simulation", maze_size)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut texture_context = window.create_texture_context();

    window.set_max_fps(30);

    let mut debugs = Vec::new();

    while let Some(event) = window.next() {
        if let Some(r) = event.render_args() {

            let mut new_debugs = debug_rx.try_iter().collect();
            debugs.append(&mut new_debugs);

            window.draw_2d(&event, |context, graphics, _device| {
                clear([1.0; 4], graphics);

                let transform = context
                    .transform
                    .trans(0.0, (maze_size.1 as f64))
                    .scale(config.pixels_per_mm as f64, -config.pixels_per_mm as f64);

                if let Some(debug) = debugs.last() {

                    let cell_width = config.simulation.mouse.map.maze.cell_width;
                    let wall_width = config.simulation.mouse.map.maze.wall_width;

                    // Draw the posts
                    for x in 0..WIDTH + 1 {
                        for y in 0..HEIGHT + 1 {
                            rectangle(
                                config.post_color,
                                [
                                    (x as f32 * cell_width - wall_width / 2.0) as f64,
                                    (y as f32 * cell_width - wall_width / 2.0) as f64,
                                    wall_width as f64,
                                    wall_width as f64,
                                ],
                                transform,
                                graphics,
                            )
                        }
                    }

                    // Draw the horizontal walls
                    for x in 0..WIDTH {
                        for y in 0..HEIGHT + 1 {
                            let edge_index = EdgeIndex {
                                x,
                                y,
                                horizontal: true,
                            };

                            let edge = debug
                                .mouse_debug
                                .map
                                .maze
                                .get_edge(edge_index)
                                .unwrap_or(&Edge::Closed);

                            let color = match edge {
                                Edge::Open => config.wall_open_color,
                                Edge::Closed => config.wall_closed_color,
                                Edge::Unknown => config.wall_unknown_color,
                            };

                            let border = debug.mouse_debug.map.front_edge.and_then(|e_i| {
                                if edge_index == e_i {
                                    Some(Border {
                                        color: config.wall_front_border_color,
                                        radius: 2.0,
                                    })
                                } else {
                                    None
                                }
                            });

                            Rectangle::new(color).maybe_border(border).draw(
                                [
                                    (x as f32 * cell_width + wall_width / 2.0) as f64,
                                    (y as f32 * cell_width - wall_width / 2.0) as f64,
                                    (cell_width - wall_width) as f64,
                                    wall_width as f64,
                                ],
                                &Default::default(),
                                transform,
                                graphics,
                            );
                        }
                    }

                    // Draw the vertical walls
                    for x in 0..WIDTH + 1 {
                        for y in 0..HEIGHT {
                            let edge_index = EdgeIndex {
                                x,
                                y,
                                horizontal: false,
                            };
                            let edge = debug
                                .mouse_debug
                                .map
                                .maze
                                .get_edge(edge_index)
                                .unwrap_or(&Edge::Closed);

                            let color = match edge {
                                Edge::Open => config.wall_open_color,
                                Edge::Closed => config.wall_closed_color,
                                Edge::Unknown => config.wall_unknown_color,
                            };

                            let border = debug.mouse_debug.map.front_edge.and_then(|e_i| {
                                if edge_index == e_i {
                                    Some(Border {
                                        color: config.wall_front_border_color,
                                        radius: 2.0,
                                    })
                                } else {
                                    None
                                }
                            });

                            Rectangle::new(color).maybe_border(border).draw(
                                [
                                    (x as f32 * cell_width - wall_width / 2.0) as f64,
                                    (y as f32 * cell_width + wall_width / 2.0) as f64,
                                    wall_width as f64,
                                    (cell_width - wall_width) as f64,
                                ],
                                &Default::default(),
                                transform,
                                graphics,
                            );
                        }
                    }

                    // Draw the path
                    if let Some(path) = &debug.mouse_debug.path.path {
                        for segment in path {
                            match segment {
                                &Segment::Line(l1, l2) => line(
                                    config.path_color,
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
                                        let t_end = t_start + t;
                                        (t_end, t_start)
                                    } else {
                                        let t_start = f32::atan2(v.y, v.x);
                                        let t_end = t_start + t;
                                        (t_start, t_end)
                                    };

                                    circle_arc(
                                        config.path_color,
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

                    // Draw the mouse
                    rectangle(
                        config.mouse_color,
                        [
                            (-config.simulation.mouse.mechanical.length / 2.0) as f64,
                            (-config.simulation.mouse.mechanical.width / 2.0) as f64,
                            config.simulation.mouse.mechanical.length as f64,
                            config.simulation.mouse.mechanical.width as f64,
                        ],
                        orientation_transform(&debug.orientation, transform),
                        graphics,
                    );
                }
            });
        }
    }

    cmd_tx.send(GuiCmd::Exit);
}