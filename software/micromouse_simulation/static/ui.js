console.log("Starting");

const MAZE_WIDTH = 16;
const MAZE_HEIGHT = 16;

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
        wall_width: 12.0,
    },
};

const MOUSE_2020_MOTION = {
    max_delta_power: 1.0,
    max_wheel_power: 0.6,
};

const initial_simulation_config = {
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

const initial_remote_config = {
    mouse: {
        mechanical: MOUSE_2019_MECH,
        path: MOUSE_SIM_PATH,
        map: MOUSE_MAZE_MAP,
        motion: MOUSE_2020_MOTION,
    },
};

function Simulation() {
    let self = this;

    self.STATE_LOADING = "loading";
    self.STATE_DISCONNECTED = "disconnected";
    self.STATE_CONNECTING = "connecting";
    self.STATE_STOPPED = "stopped";
    self.STATE_RUNNING = "running";

    let worker = new Worker('worker.js');

    self.state = self.STATE_LOADING;

    self.onupdate = function() {};

    let last_time = 0;
    let do_update = function(time) {
        if (time - last_time > 33) {
            self.onupdate();

            last_time = time;
        }
    };

    self.debugs = [];
    self.index = -1;
    self.debug = function() {
        if (self.index < 0) {
            return self.debugs[self.debugs.length-1]
        } else {
            return self.debugs[self.index]
        }
    };

    self.graphs = [];

    self.connect = function(type, config, options) {
        worker.postMessage({
            name: 'connect',
            data: {
                type: type,
                config: config,
                options: options,
            },
        });
    };

    self.disconnect = function() {
        worker.postMessage({
            name: 'disconnect',
            data: null,
        });
    };

    self.start = function() {
        worker.postMessage({
            name: 'start',
            data: null,
        });
    };

    self.stop = function() {
        worker.postMessage({
            name: 'stop',
            data: null,
        });
        self.running = false;
    };

    self.reset = function() {
        worker.postMessage({
            name: 'reset',
            data: null,
        })
    };

    self.update = function() {
        requestAnimationFrame(do_update);
    };

    self.send_config = function(config) {
        worker.postMessage({
            name: 'config',
            data: config,
        });
    };

    worker.onmessage = function(event) {
        let msg = event.data;
        if (msg.name === "loaded") {
            self.state = self.STATE_LOADING
        } else if (msg.name === "disconnected") {
            self.state = self.STATE_DISCONNECTED;
        } else if (msg.name === "connecting") {
            self.state = self.STATE_CONNECTING;
        } else if (msg.name === "connected") {
            self.debugs = [];
            self.index = -1;
            self.state = self.STATE_STOPPED;
        } else if (msg.name === "running") {
            self.state = self.STATE_RUNNING;
        } else if (msg.name === "stopped") {
            self.state = self.STATE_STOPPED;
        } else if (msg.name === 'reset') {
            self.debugs = [];
            self.index = -1;
        } else if (msg.name === "debug") {
            if (msg.data.time < self.debugs[self.debugs.length]) {
                self.debugs = [];
                self.index = -1;
            }
            self.debugs.push(msg.data)
        }

        requestAnimationFrame(do_update);
    };
}

function run_worker() {
    if (window.Worker) {
        let root = document.getElementById('ui');

        let simulation = new Simulation();
        let ui = new Ui(root, simulation);
        simulation.onupdate = function() {
            ui.update(simulation);
        };
    }
}

function Ui(parent, state) {
    let self = this;

    let debug = div().classes('column is-narrow').style('width', '27em');
    let maze = div().classes('column is-5');
    let graph = div().classes('column');

    self.root = div().classes('columns is-multiline').style("margin", "1em").children([
        debug,
        maze,
        graph,
    ]);
    parent.append(self.root.el);

    self.setup_ui = new SetupUi(debug.el, state);
    self.control_ui = new ControlUi(debug.el, state);
    self.debug_ui = new DebugUi(debug.el, state);
    self.config_ui = new ConfigUi(debug.el, state);
    self.maze_ui = new MazeUi(maze.el, state);
    self.graph_ui = new GraphUi(graph.el, state);

    self.update = function(state) {
        self.control_ui.update(state);
        self.maze_ui.update(state);
        self.graph_ui.update(state);
        self.debug_ui.update(state);
    }
}



run_worker();
