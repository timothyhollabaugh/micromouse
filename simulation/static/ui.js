// Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init, { JsSimulation } from './pkg/simulation.js';

async function run() {
    await init();

    const MOUSE_2019_MECH = {
        wheel_diameter: 32.0,
        gearbox_ratio: 75.0,
        ticks_per_rev: 12.0,
        wheelbase: 74.0,
        width: 64.0,
        length: 90.0,
        front_offset: 48.0,
    };

    const MOUSE_SIM_PATH = {
        p: 10.0,
        i: 0.0,
        d: 0.0,
        offset_p: 0.002,
    };

    const MOUSE_MAZE_MAP = {
        maze: {
            cell_width: 180.0,
            wall_width: 20.0,
        },
    };

    const MOUSE_2020_MOTION = {
        max_wheel_delta_power: 10000.0,
    };

    const config ={
        mouse: {
            mechanical: MOUSE_2019_MECH,
            path: MOUSE_SIM_PATH,
            map: MOUSE_MAZE_MAP,
            motion: MOUSE_2020_MOTION,
        },

        max_speed: 500.0,

        initial_orientation: {
            position: {
                x: 1000.0,
                y: 1000.0,
            },
            direction: 0.0,
        },

        millis_per_step: 10,
        max_wheel_accel: 60000.0,

    };

    // And afterwards we can use all the functionality defined in wasm.
    let simulation = new JsSimulation(config);

    let time_p = document.getElementById("time");

    setInterval(function() {
        let debug = simulation.update(config);
        time_p.innerText = debug.time;
    }, config.millis_per_step);
}

run();
