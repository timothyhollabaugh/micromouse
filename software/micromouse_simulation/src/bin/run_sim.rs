use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::exit;
use std::time::{Duration, Instant};

use typenum::consts::U2048;

use micromouse_logic::comms::{DebugMsg, DebugPacket};
use micromouse_logic::config::sim::MOUSE_2019;
use micromouse_logic::fast::{Orientation, Vector, DIRECTION_PI_2};
use micromouse_logic::slow::maze::Maze;
use micromouse_logic::slow::MazeOrientation;
use micromouse_simulation::simulation::{Simulation, SimulationConfig};

pub fn main() {
    let args: Vec<_> = env::args().collect();
    println!("{:?}", args);

    let maze_file_name = args.get(1).expect("No maze file provided");

    println!("Using maze: {}", maze_file_name);

    let mut maze_file = File::open(maze_file_name).expect("Could not open maze file");

    let mut file_bytes = [0; 256];

    maze_file.read_exact(&mut file_bytes).unwrap();

    let maze = Maze::from_file(file_bytes);

    let config = SimulationConfig {
        mouse: MOUSE_2019,
        millis_per_step: 10,
        millis_per_sensor_update: 20,
        initial_orientation: Orientation {
            position: Vector {
                x: 0.5 * 180.0,
                y: 0.5 * 180.0,
            },
            direction: DIRECTION_PI_2,
        },
        max_wheel_accel: 1.0,
        max_speed: 1.0,
        maze,
    };

    let mut simulation = Simulation::new(&config);

    let mut debugs = Vec::new();

    let result = loop {
        let debug = simulation.update(&config);

        println!("Ran sim at time {}", debug.mouse.time);

        debugs.push(debug.clone());

        if debug.mouse.time > 1000 * 60 * 10 {
            break Err(());
        }

        let position = debug.mouse.maze_orientation.position;

        if (position.x == 7 || position.x == 8) && (position.y == 7 || position.y == 8) {
            break Ok(debug.mouse.time);
        }
    };

    let mut outfile = File::create("out.dat").expect("Could not create out file");

    for (count, debug) in debugs.iter().enumerate() {
        let mut msgs = heapless::Vec::new();

        msgs.push(DebugMsg::Orientation(debug.mouse.orientation.clone()))
            .ok();
        msgs.push(DebugMsg::Hardware(debug.mouse.hardware.clone()))
            .ok();
        msgs.push(DebugMsg::Slow(debug.mouse.slow.clone())).ok();
        msgs.push(DebugMsg::Localize(debug.mouse.localize.clone()))
            .ok();
        msgs.push(DebugMsg::MotionQueue(debug.mouse.motion_queue.clone()))
            .ok();
        msgs.push(DebugMsg::MotorControl(
            debug.mouse.motion_control.motor_control,
        ))
        .ok();
        msgs.push(DebugMsg::MotionHandler(
            debug.mouse.motion_control.handler.clone(),
        ))
        .ok();

        let packet = DebugPacket {
            msgs,
            battery: 5000,
            time: debug.mouse.time,
            delta_time_sys: config.millis_per_step,
            delta_time_msg: config.millis_per_step,
            count: count as u16,
        };

        let bytes =
            postcard::to_vec::<U2048, _>(&packet).expect("Could not serialize debug");

        outfile
            .write_all(&bytes)
            .expect("Could not write data to file");
    }

    if let Ok(ms) = result {
        println!("time: {} ms", ms);
    } else {
        println!("time: timed out");
    }
}
