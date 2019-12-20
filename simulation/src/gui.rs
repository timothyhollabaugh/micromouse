use std::time::Duration;
use std::time::Instant;

use std::thread;

use std::f32::consts::PI;

use std::io::BufReader;

use crossbeam::channel;

use druid::kurbo::Affine;
use druid::kurbo::Rect;
use druid::kurbo::Size;
use druid::kurbo::Vec2;
use druid::piet::Color;
use druid::piet::RenderContext;
use druid::widget::Align;
use druid::widget::Button;
use druid::widget::Flex;
use druid::widget::Label;
use druid::widget::Padding;
use druid::widget::WidgetExt;
use druid::Data;
use druid::LifeCycle;
use druid::LocalizedString;
use druid::Widget;
use druid::WindowDesc;
use druid::{
    AppLauncher, BaseState, BoxConstraints, Env, Event, EventCtx, LayoutCtx, PaintCtx, UpdateCtx,
};

use mouse::config::MechanicalConfig;
use mouse::maze::Edge;
use mouse::maze::EdgeIndex;
use mouse::maze::Maze;
use mouse::maze::HEIGHT as MAZE_HEIGHT;
use mouse::maze::WIDTH as MAZE_WIDTH;

use mouse::map::Orientation;
use mouse::map::Vector;
use mouse::path::Segment;

use crate::simulation::RemoteMouse;
use crate::simulation::Simulation;
use crate::simulation::SimulationConfig;
use crate::simulation::SimulationDebug;

fn into_color(color: [f32; 4]) -> Color {
    Color::rgba(color[0], color[1], color[2], color[3])
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GuiConfig {
    pub simulation: SimulationConfig,
    pub pixels_per_mm: f32,
    pub time_scale: f32,
    pub simulated_mouse_color: [f32; 4],
    pub real_mouse_color: [f32; 4],
    pub path_color: [f32; 4],
    pub maze_color: [f32; 4],
    pub wall_open_color: [f32; 4],
    pub wall_closed_color: [f32; 4],
    pub wall_unknown_color: [f32; 4],
    pub wall_err_color: [f32; 4],
    pub wall_front_border_color: [f32; 4],
    pub wall_left_border_color: [f32; 4],
    pub wall_right_border_color: [f32; 4],
    pub post_color: [f32; 4],
}

impl GuiConfig {
    pub fn pixels_per_cell(&self) -> f32 {
        self.simulation.mouse.map.maze.cell_width * self.pixels_per_mm
    }

    pub fn pixels_per_wall(&self) -> f32 {
        self.simulation.mouse.map.maze.wall_width * self.pixels_per_mm
    }

    pub fn maze_pixel_size(&self) -> (f32, f32) {
        (
            self.pixels_per_cell() * MAZE_WIDTH as f32 + self.pixels_per_wall(),
            self.pixels_per_cell() * MAZE_HEIGHT as f32 + self.pixels_per_wall(),
        )
    }

    pub fn maze_mm_size(&self) -> (f32, f32) {
        let maze_config = self.simulation.mouse.map.maze;
        (
            maze_config.cell_width * MAZE_WIDTH as f32 + maze_config.wall_width,
            maze_config.cell_width * MAZE_HEIGHT as f32 + maze_config.wall_width,
        )
    }
}

/*
fn orientation_transform<T: Transformed + Sized>(orientation: &Orientation, transform: T) -> T {
    transform
        .trans(orientation.position.x as f64, orientation.position.y as f64)
        .rot_rad(orientation.direction.into())
}
*/

enum GuiCmd {
    Exit,
}

pub fn run(config: GuiConfig) {
    let (debug_tx, debug_rx) = channel::unbounded();
    let (cmd_tx, cmd_rx) = channel::unbounded();

    let simulation_thread = thread::spawn(move || run_simulation(debug_tx, cmd_rx, &config));
    let gui_thread = thread::spawn(move || run_gui(debug_rx, cmd_tx, &config.clone()));

    simulation_thread.join().ok();
    gui_thread.join().ok();
}

fn run_simulation(
    debug_tx: channel::Sender<SimulationDebug>,
    cmd_rx: channel::Receiver<GuiCmd>,
    config: &GuiConfig,
) {
    let mut simulation = Simulation::new(&config.simulation, 0);

    //let serial = serialport::open("/dev/rfcomm0").unwrap();
    //let mut simulation = RemoteMouse::new(&config.simulation, serial);

    'main: loop {
        for cmd in cmd_rx.try_iter() {
            match cmd {
                GuiCmd::Exit => break 'main,
            }
        }

        let debug = simulation.update(&config.simulation);

        debug_tx.send(debug).ok();

        thread::sleep(Duration::from_millis(
            (config.simulation.millis_per_step as f32 * config.time_scale) as u64,
        ));
    }
}

/*
fn edge_border(
    current_edge: EdgeIndex,
    other_edge: Option<EdgeIndex>,
    color: [f32; 4],
) -> Option<Border> {
    other_edge.and_then(|other_edge| {
        if current_edge == other_edge {
            Some(Border { color, radius: 4.0 })
        } else {
            None
        }
    })
}
*/

#[derive(Data, Clone)]
struct GuiData {
    #[druid(same_fn = "PartialEq::eq")]
    debug: SimulationDebug,

    #[druid(same_fn = "PartialEq::eq")]
    config: GuiConfig,

    #[druid(ignore)]
    rx: channel::Receiver<SimulationDebug>,

    #[druid(ignore)]
    tx: channel::Sender<GuiCmd>,
}

fn run_gui(
    debug_rx: channel::Receiver<SimulationDebug>,
    cmd_tx: channel::Sender<GuiCmd>,
    config: &GuiConfig,
) {
    let maze_size = config.maze_pixel_size();
    let main_window =
        WindowDesc::new(ui_main).window_size((maze_size.0 as f64, maze_size.1 as f64 + 32.0));
    let data = GuiData {
        debug: Default::default(),
        config: *config,
        rx: debug_rx,
        tx: cmd_tx,
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("Could not launch app");
}

fn ui_main() -> impl Widget<GuiData> {
    //let text = LocalizedString::new("hello-counter")
    //.with_arg("count", |data: &GuiData, _env| data.debug.time.into());

    //let label = Label::new(text).padding(5.0).center();
    let label = Label::new(|data: &GuiData, _env: &Env| format!("Time: {}", data.debug.time))
        .padding(5.0)
        .center();
    let channel_widget = ChannelWidget::<GuiData, SimulationDebug>::new(
        |data: &GuiData, _env: &Env| data.rx.clone(),
        |debug: SimulationDebug, _ctx: &mut EventCtx, data: &mut GuiData, _env: &Env| {
            data.debug = debug
        },
    );

    let maze_widget = MazeWidget::new(
        |data: &GuiData, _env| data.debug.clone(),
        |data: &GuiData, _env| data.config,
    );

    let mut col = Flex::column();
    col.add_child(label, 1.0);
    col.add_child(maze_widget, 0.0);
    col.add_child(channel_widget, 0.0);
    col
}

struct MazeWidget<T> {
    debug: Box<dyn Fn(&T, &Env) -> SimulationDebug>,
    config: Box<dyn Fn(&T, &Env) -> GuiConfig>,
}

impl<T> MazeWidget<T> {
    pub fn new(
        debug: impl Fn(&T, &Env) -> SimulationDebug + 'static,
        config: impl Fn(&T, &Env) -> GuiConfig + 'static,
    ) -> MazeWidget<T> {
        MazeWidget {
            debug: Box::new(debug),
            config: Box::new(config),
        }
    }
}

impl<T: Data> Widget<T> for MazeWidget<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {}

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: Option<&T>, data: &T, env: &Env) {}

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let config = (self.config)(data, env);
        bc.constrain((
            config.maze_pixel_size().0 as f64,
            config.maze_pixel_size().1 as f64,
        ))
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &T, env: &Env) {
        let debug = (self.debug)(data, env);
        let config = (self.config)(data, env);
        let maze_config = config.simulation.mouse.map.maze;

        // Scale transform so 1px = 1mm
        paint_ctx.transform(Affine::scale(config.pixels_per_mm as f64));

        let maze_size = config.maze_mm_size();

        // Fill the background
        paint_ctx.fill(
            Rect::new(0.0, 0.0, maze_size.0 as f64, maze_size.1 as f64),
            &into_color(config.maze_color),
        );

        // Draw the maze
        for i in 0..MAZE_WIDTH + 1 {
            for j in 0..MAZE_HEIGHT + 1 {
                let x = i as f64 * maze_config.cell_width as f64;
                let y = j as f64 * maze_config.cell_width as f64;

                // Draw the posts
                paint_ctx.fill(
                    Rect::new(
                        x,
                        y,
                        x + maze_config.wall_width as f64,
                        y + maze_config.wall_width as f64,
                    ),
                    &into_color(config.post_color),
                );

                // Draw the horizontal walls
                if i <= MAZE_WIDTH {
                    draw_wall(config, &debug.mouse_debug.map.maze, i, j, true, paint_ctx);
                }

                // Draw the vertical walls
                if j <= MAZE_HEIGHT {
                    draw_wall(config, &debug.mouse_debug.map.maze, i, j, false, paint_ctx);
                }
            }
        }

        // Draw the mouse
        draw_mouse(
            paint_ctx,
            &config.simulation.mouse.mechanical,
            debug.mouse_debug.orientation,
            into_color(config.simulated_mouse_color),
        );

        draw_mouse(
            paint_ctx,
            &config.simulation.mouse.mechanical,
            debug.orientation,
            into_color(config.real_mouse_color),
        );
    }
}

fn draw_mouse(
    paint_ctx: &mut PaintCtx,
    mech: &MechanicalConfig,
    orientation: Orientation,
    color: Color,
) {
    paint_ctx
        .with_save(|paint_ctx| {
            paint_ctx.transform(Affine::translate((
                orientation.position.x as f64,
                orientation.position.y as f64,
            )));

            paint_ctx.transform(Affine::rotate(f32::from(orientation.direction) as f64));

            paint_ctx.fill(
                Rect::new(
                    mech.front_offset as f64 - mech.length as f64,
                    -mech.width as f64 / 2.0,
                    mech.front_offset as f64,
                    mech.width as f64 / 2.0,
                ),
                &color,
            );
            Ok(())
        })
        .ok();
}

fn draw_wall(
    config: GuiConfig,
    maze: &Maze,
    i: usize,
    j: usize,
    horizontal: bool,
    paint_ctx: &mut PaintCtx,
) {
    let maze_config = config.simulation.mouse.map.maze;
    let index = EdgeIndex {
        x: i,
        y: j,
        horizontal,
    };

    let wall = maze.get_edge(index);

    let color = match wall {
        // The top/bottom border for horizontal walls
        _ if horizontal && (j == 0 || j == MAZE_HEIGHT) => config.wall_closed_color,

        // The left/right border for vertical walls
        _ if !horizontal && (i == 0 || i == MAZE_WIDTH) => config.wall_closed_color,

        // Closed walls in the middle
        Some(Edge::Closed) => config.wall_closed_color,

        // Open walls in the middle
        Some(Edge::Open) => config.wall_open_color,

        // Unknown walls in the middle
        Some(Edge::Unknown) => config.wall_unknown_color,

        // If the index is outside the maze
        None => config.wall_err_color,
    };

    let x = i as f64 * maze_config.cell_width as f64;
    let y = j as f64 * maze_config.cell_width as f64;

    let rect = if horizontal {
        Rect::new(
            x + maze_config.wall_width as f64,
            y,
            x + maze_config.cell_width as f64,
            y + maze_config.wall_width as f64,
        )
    } else {
        Rect::new(
            x,
            y + maze_config.wall_width as f64,
            x + maze_config.wall_width as f64,
            y + maze_config.cell_width as f64,
        )
    };

    paint_ctx.fill(rect, &into_color(color));
}

/// A widget that controls the simulation. It handles talking to the simulation through a channel
struct ChannelWidget<T, Rx> {
    channel: Box<dyn Fn(&T, &Env) -> channel::Receiver<Rx>>,
    on_recv: Box<dyn Fn(Rx, &mut EventCtx, &mut T, &Env)>,
}

impl<T, Rx> ChannelWidget<T, Rx> {
    pub fn new(
        channel: impl Fn(&T, &Env) -> channel::Receiver<Rx> + 'static,
        on_recv: impl Fn(Rx, &mut EventCtx, &mut T, &Env) + 'static,
    ) -> ChannelWidget<T, Rx> {
        ChannelWidget {
            channel: Box::new(channel),
            on_recv: Box::new(on_recv),
        }
    }
}

impl<T: Data, Rx> Widget<T> for ChannelWidget<T, Rx> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::LifeCycle(LifeCycle::WindowConnected) => {
                println!("Window Connected!");
                ctx.request_anim_frame()
            }
            Event::AnimFrame(_delta_nanos) => {
                let rx = (self.channel)(data, env);

                for d in rx.try_iter() {
                    (self.on_recv)(d, ctx, data, env)
                }

                ctx.invalidate();
                ctx.request_anim_frame();
            }
            _ => {}
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: Option<&T>, data: &T, env: &Env) {}

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        Size::new(0.0, 0.0)
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, base_state: &BaseState, data: &T, env: &Env) {
        println!("Painting");
    }
}

/*
fn run_gui(
    debug_rx: mpsc::Receiver<SimulationDebug>,
    cmd_tx: mpsc::Sender<GuiCmd>,
    config: &GuiConfig,
) {
    let maze_size = (
        (WIDTH as f32 * config.pixels_per_cell()) as u32,
        (HEIGHT as f32 * config.pixels_per_cell()) as u32,
    );

    let ui_width = 600;
    let ui_height = maze_size.1;

    let window_size = (maze_size.0, maze_size.1 + ui_width);

    let mut window: PistonWindow = WindowSettings::new("Micromouse Simulation", window_size)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut ui = UiBuilder::new([ui_width as f64, ui_height as f64]).build();

    ui.fonts.insert_from_file("FiraSans-Regular.ttf");

    let mut text_vertex_data = Vec::new();
    let (mut glyph_cache, mut text_texture_cache) = {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = GlyphCache::builder()
            .dimensions(ui_width, ui_height)
            .scale_tolerance(SCALE_TOLERANCE)
            .position_tolerance(POSITION_TOLERANCE)
            .build();
        let buffer_len = ui_width as usize * ui_height as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let factory = &mut window.factory;
        let texture =
            G2dTexture::from_memory_alpha(factor, &init, ui_width, ui_height, &settings).unwrap();
        (cache, texture)
    };

    //let ids = Ids::new(ui.widget_id_generator());

    let mut app = MouseGui::new();

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

                            let front_edge_border = edge_border(
                                edge_index,
                                debug.mouse_debug.map.front_edge,
                                config.wall_front_border_color,
                            );
                            let left_edge_border = edge_border(
                                edge_index,
                                debug.mouse_debug.map.left_edge,
                                config.wall_left_border_color,
                            );
                            let right_edge_border = edge_border(
                                edge_index,
                                debug.mouse_debug.map.right_edge,
                                config.wall_right_border_color,
                            );

                            let edge_border =
                                front_edge_border.or(left_edge_border).or(right_edge_border);

                            Rectangle::new(color).maybe_border(edge_border).draw(
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

                            let front_edge_border = edge_border(
                                edge_index,
                                debug.mouse_debug.map.front_edge,
                                config.wall_front_border_color,
                            );
                            let left_edge_border = edge_border(
                                edge_index,
                                debug.mouse_debug.map.left_edge,
                                config.wall_left_border_color,
                            );
                            let right_edge_border = edge_border(
                                edge_index,
                                debug.mouse_debug.map.right_edge,
                                config.wall_right_border_color,
                            );

                            let edge_border =
                                front_edge_border.or(left_edge_border).or(right_edge_border);

                            Rectangle::new(color).maybe_border(edge_border).draw(
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
                        config.simulated_mouse_color,
                        [
                            (-config.simulation.mouse.mechanical.length / 2.0) as f64,
                            (-config.simulation.mouse.mechanical.width / 2.0) as f64,
                            config.simulation.mouse.mechanical.length as f64,
                            config.simulation.mouse.mechanical.width as f64,
                        ],
                        orientation_transform(&debug.orientation, transform),
                        graphics,
                    );

                    // Draw the mouse
                    Rectangle::new([0.0, 0.0, 0.0, 0.0])
                        .border(Border {
                            color: config.real_mouse_color,
                            radius: 4.0,
                        })
                        .draw(
                            [
                                (-config.simulation.mouse.mechanical.length / 2.0) as f64,
                                (-config.simulation.mouse.mechanical.width / 2.0) as f64,
                                config.simulation.mouse.mechanical.length as f64,
                                config.simulation.mouse.mechanical.width as f64,
                            ],
                            &Default::default(),
                            orientation_transform(&debug.mouse_debug.orientation, transform),
                            graphics,
                        );
                }
            });
        }
    }

    cmd_tx.send(GuiCmd::Exit);
}
*/
