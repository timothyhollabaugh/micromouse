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
    max_wheel_delta_power: 10000.0,
};

const initial_config = {
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

function UiState(send) {
    let self = this;

    self.send_config = function(config) {
        send({name: "config", data: config})
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
    self.running = false;
    self.start = function() {
        if (!self.running) {
            send({name: 'start', data: {}});
            self.running = true;
        }
    };
    self.stop = function() {
        if (self.running) {
            send({name: 'stop', data: {}});
            self.running = false;
        }
    };
    self.reset = function() {
        send({name: 'reset', data: {}});
        self.debugs = [];
    };

    self.graphs = [];
}

function run_worker(config) {
    if (window.Worker) {
        let worker = new Worker('worker.js');
        worker.postMessage({name: 'config', data: config});

        let ui_state = new UiState(function(m) { worker.postMessage(m) } );

        worker.onmessage = function (event) {
            ui_state.debugs.push(event.data);
        };

        let root = document.getElementById('ui');

        let ui = new Ui(root, ui_state);

        let last_time = 0;

        function simulate(time) {
            if (time - last_time > 33 ){
                ui.update(ui_state);
                last_time = time;
            }
            requestAnimationFrame(simulate);
        }

        requestAnimationFrame(simulate);
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

    self.state_ui = new StateUi(debug.el, state);
    self.debug_ui = new DebugUi(debug.el, state);
    self.config_ui = new ConfigUi(debug.el, state);
    self.maze_ui = new MazeUi(maze.el, state);
    self.graph_ui = new GraphUi(graph.el, state);

    self.update = function(state) {
        self.state_ui.update(state);
        self.maze_ui.update(state);
        self.graph_ui.update(state);
        self.debug_ui.update(state);
    }
}

function StateUi(parent, state) {
    let self = this;

    let controls = fieldset().classes('control field has-addons').disabled(true).children([
        div().classes('control').children([
            input()
                .type('number')
                .classes('input')
                .style('text-align', 'right')
                .style('font-family', 'monospace')
                .style('width', '7em')
                .oninput(function(){
                    if (!state.running && this.el.value > 0 && this.el.value < state.debugs.length) {
                        state.index = Number(this.el.value);
                    }
                })
                .onupdate(function(state) {
                    if (state.running) {
                        this.value(state.debugs.length);
                    }
                })
        ]),
        div().classes('control').children([
            button()
                .classes('button is-static')
                .text('/ 0')
                .style('font-family', 'monospace')
                .onupdate(function(state) {
                    this.text('/ ' + (state.debugs.length-1))
                })
        ])
    ]);

    let root = div().classes("card").style("margin-bottom", "1em").children([
        p().classes("card-header").children([
            p().classes("card-header-title").text("State"),
        ]),
        div().classes("card-content").children([
            div().classes("content").children([
                div().classes('field is-grouped').children([
                    button().classes('control button is-primary').text('Start').style('width', '4em').onclick(function () {
                        if (state.running) {
                            state.stop();
                            controls.disabled(false);
                            this.text('Start');
                        } else {
                            state.start();
                            state.index = -1;
                            controls.disabled(true);
                            this.text('Stop');
                        }
                    }),
                    button().classes('control button is-danger').text('Reset').style('width', '4em').onclick(function() {
                        state.reset()
                    }),
                    controls,
                ])
            ])
        ])
    ]);

    parent.append(root.el);

    self.update = function(state) {
        root.update(state);
    }
}

function DebugUi(parent) {
    let self = this;

    let content = div().classes("content");

    let root = div().classes("card").style("margin-bottom", "1em").children([
        p().classes("card-header").children([
            p().classes("card-header-title").text("Debug"),
        ]),
        div().classes("card-content").children([content])
    ]);

    parent.append(root.el);

    let node = new Node('debug', function(debug) { return debug });
    content.el.append(node.root);

    self.update = function(state) {
        node.update(state);
    }
}

function Node(path, f) {
    let self = this;

    self.root = document.createElement('div');

    let header = document.createElement('div');
    self.root.append(header);

    let name = document.createElement('span');
    let paths = path.split('/');
    name.innerText = paths[paths.length-1];
    header.append(name);

    let value = document.createElement('span');
    value.className += 'is-pulled-right';
    value.style.fontFamily = 'monospace';
    value.style.width = '6em';
    header.append(value);

    let icon = null;
    let nodes = {};
    let olddata = null;
    let open = false;
    let children = null;
    let graphcheck = null;

    self.update = function(state) {
        let data = f(state.debug());
        if (data !== null && typeof data === 'object') {
            if (!header.onclick) {
                header.onclick = function() {
                    if (open) {
                        open = false;
                        icon.innerHTML = feather.icons['chevron-right'].toSvg({height: '1em'});
                    } else {
                        open = true;
                        icon.innerHTML = feather.icons['chevron-down'].toSvg({height: '1em'});
                    }
                };
                header.style.cursor = 'pointer';
            }
            if (!icon) {
                icon = document.createElement('span');
                icon.innerHTML = feather.icons['chevron-right'].toSvg({height: '1em'});
                header.prepend(icon);
            }
            if (open) {
                if (!children) {
                    children = document.createElement('div');
                    children.style.paddingLeft = '0.5em';
                    children.style.marginLeft = '0.5em';
                    children.style.borderLeft = 'solid black 1px';
                    self.root.append(children);
                }
                for (let key in data) {
                    if (data.hasOwnProperty(key)) {
                        if (nodes[key]) {
                            nodes[key].update(state)
                        } else {
                            let node = new Node(path + "/" + key, function(debug) { return f(debug)[key] });
                            node.update(state);
                            nodes[key] = node;
                            children.append(node.root);
                        }
                    }
                }
            } else {
                if (children) {
                    children.remove();
                    children = undefined;
                }

                if (nodes !== {}) {
                    nodes = {};
                }

                if (olddata) {
                    olddata = null;
                }
            }
            value.innerText = Object.keys(data).length + " items";
        } else if (data !== undefined) {
            if (olddata !== data) {
                if (typeof data === 'number') {
                    value.innerText = math.format(data, {precision: 4, upperExp: 4});
                } else if (typeof data === 'string') {
                    value.innerText = data;
                } else {
                    value.innerText = String(data);
                }
                olddata = data;
            }

            if (!graphcheck) {
                graphcheck = document.createElement('input');
                graphcheck.type = "checkbox";
                graphcheck.className += 'is-pulled-right';
                graphcheck.style.marginRight = "1em";
                graphcheck.onchange = function() {
                    if (graphcheck.checked) {
                        state.graphs[path] = f;
                    } else {
                        delete state.graphs[path];
                    }
                }
                if (path in state.graphs) {
                    graphcheck.checked = true;
                }
                header.append(graphcheck);
            }
        }
    };
}

function ConfigUi(parent, state) {
    let self = this;

    let local_config = initial_config;

    let content = div().classes("content");

    let root = div().classes("panel").style("margin-bottom", "1em").children([
        p().classes("card-header").children([
            p().classes("card-header-title").text("Config"),
        ]),
        div().classes("card-content").children([content]),
        div().classes("card-footer").children([
            button().classes("button card-footer-item is-primary").text("Set Config").onclick(function() {
                console.log(local_config);
                state.send_config(local_config);
            })
        ])
    ]);

    parent.append(root.el);

    let node = new ConfigNode('config', initial_config, function(c) {
        local_config = c;
    });
    content.el.append(node.root);
}

function ConfigNode(key, initial_value, f) {
    let self = this;

    self.root = document.createElement('div');

    if (initial_value !== null && typeof initial_value === 'object') {

        let name = document.createElement('span');
        name.innerText = key;
        self.root.append(name);

        let nodes = {};
        let value = initial_value;

        let children = document.createElement('div');
        children.style.paddingLeft = '0.5em';
        children.style.marginLeft = '0.5em';
        children.style.borderLeft = 'solid grey 1px';

        for (let ckey in initial_value) {
            if (initial_value.hasOwnProperty(ckey)) {
                let node = new ConfigNode(ckey, initial_value[ckey], function(v) {
                    value[ckey] = v;
                    f(value)
                });
                nodes[ckey] = node;
                children.append(node.root);
            }
        }

        self.root.append(children);
    } else if (initial_value !== undefined) {

        let value = input()
            .classes("input is-pulled-right is-small")
            .style("font-family", "monospace")
            .style("width", "6em");

        if (typeof initial_value === 'number') {
            value.type('number');
            value.value(initial_value);
            value.oninput(function() {
                f(Number(value.el.value))
            })
        } else {
            value.value(String(initial_value));
            value.oninput(function(e) {
                    f(value.el.value)
                });
        }

        let header = div().classes("field is-horizontal").children([
            div().classes("field-label").children([
                label().classes("label").style("font-weight", "400").text(key),
            ]),
            div().classes("field-body").children([
                div().classes("field").children([
                    p().classes("control").children([value])
                ])
            ])
        ]);

        self.root.append(header.el);
    }
}

function GraphUi(parent, state) {
    let self = this;

    let range = 1000;

    let content = div().classes("card-content").children([
        div().classes('level').children([
            div().classes('level-left has-text-centered').children([
                p().classes('level-item').text("Graphs"),
            ]),
            div().classes('level-right').children([
                div().classes('level-item field has-addons').children([
                    div().classes('control').children([
                        button().classes('button is-static').text("Range: "),
                    ]),
                    div().classes('control').children([
                        input()
                            .type('number')
                            .classes('input')
                            .style('text-align', 'right')
                            .style('font-family', 'monospace')
                            .style('width', '6em')
                            .value(range)
                            .oninput(function() {
                                range = Number(this.el.value);
                                console.log(range);
                            }),
                    ]),
                    div().classes('control').children([
                        button().classes('button is-static').text("steps"),
                    ]),
                ]),
            ])
        ]),
    ]);

    let root = div().classes("card").children([content]) ;
    parent.append(root.el);

    let oldgraphs = {};

    self.update = function(state) {
        for (let key in state.graphs) {
            if (state.graphs.hasOwnProperty(key)) {
                let f = state.graphs[key];
                if (!(key in oldgraphs)) {
                    oldgraphs[key] = new Graph(content.el, key)
                }
                oldgraphs[key].update(range, state, function(state, index) { return f(state.debugs[index]) })
            }
        }

        for (let key in oldgraphs) {
            if (oldgraphs.hasOwnProperty(key)) {
                if (!(key in state.graphs)) {
                    oldgraphs[key].root.el.remove();
                    delete oldgraphs[key];
                }
            }
        }
    }

}

function Graph(parent, path) {
    let self = this;

    let min = 0;
    let max = 1;

    self.root = div().children([
        div().classes('level').children([
            div().classes('level-left has-text-centered').children([
                p().classes('level-item').text(path),
            ]),
            div().classes('level-right').children([
                div().classes('level-item field is-grouped').children([
                    div().classes('control field has-addons').children([
                        div().classes('control').children([
                            button().classes('button is-static').text("Max: "),
                        ]),
                        div().classes('control').children([
                            input()
                                .type('number')
                                .classes('input')
                                .style('text-align', 'right')
                                .style('font-family', 'monospace')
                                .style('width', '6em')
                                .value(max)
                                .oninput(function() { max = Number(this.el.value); }),
                        ]),
                    ]),
                    div().classes('control field has-addons').children([
                        div().classes('control').children([
                            button().classes('button is-static').text("Min: "),
                        ]),
                        div().classes('control').children([
                            input()
                                .type('number')
                                .classes('input')
                                .style('text-align', 'right')
                                .style('font-family', 'monospace')
                                .style('width', '6em')
                                .value(min)
                                .oninput(function() { min = Number(this.el.value); }),
                        ]),
                    ]),
                ]),
            ]),
        ]),
    ]);
    parent.append(self.root.el);

    let draw = SVG(self.root.el).size("100%", 100);
    let line = draw.polyline([]).fill('none').stroke({width: 2});

    let WIDTH = draw.node.clientWidth;
    let HEIGHT = draw.node.clientHeight;

    let centerline = draw.line(WIDTH/2, 0, WIDTH/2, HEIGHT).stroke({width: 1, color: '#999999'});
    let zeroline = draw.line(0, HEIGHT/2, WIDTH, HEIGHT/2).stroke({width: 1, color: '#999999'});

    self.update = function(range, state, f) {

        let WIDTH = draw.node.clientWidth;
        let HEIGHT = draw.node.clientHeight;

        let points = [];

        let index = state.index;

        if (index < 0) {
            index = state.debugs.length;
        }

        let start = index - range;

        if (state.debugs.length > range && index > state.debugs.length - range/2) {
            start = state.debugs.length - range;
        } else if (state.debugs.length > range && index > state.debugs.length - range) {
            start = index - range/2;
        }

        if (start < 0) {
            start = 0;
        }

        for (let i = 0; i < range; i++) {
            let index = i + start;
            if (index < state.debugs.length) {
                let value = f(state, index) - min;
                points[i] = [i * WIDTH / range, HEIGHT - value * HEIGHT / (max - min)];
            }
        }

        //line.clear();
        line.plot(points);

        let center = index - start;
        centerline.plot(center * WIDTH/range, 0, center * WIDTH/range, HEIGHT);

        let zero = -min * HEIGHT / (max - min);

        if (min > 0 && max > 0) {
            zero = 0;
        }

        if (min < 0 && max < 0) {
            zero = HEIGHT;
        }

        zeroline.plot(0, HEIGHT-zero, WIDTH, HEIGHT-zero);
    }
}

function MazeUi(parent) {
    let self = this;

    const wall_open_color = '#ffffff';
    const wall_closed_color = '#444444';
    const wall_unknown_color = '#999999';
    const wall_err_color = '#ff0000';
    const mouse_int_color = '#00ff00';
    const mouse_ext_color = '#ff0000';

    let content = div().classes("card-content");

    let root = div().classes("card").children([
        div().classes("card-header").children([
            p().classes("card-header-title").text("Maze")
        ]),
        content,
    ]);

    parent.append(root.el);

    let draw = SVG(content.el);
    let world = undefined;

    function redraw(config) {

        const maze_config = config.mouse.map.maze;
        const maze_width_mm = MAZE_WIDTH * maze_config.cell_width + maze_config.wall_width;
        const maze_height_mm = MAZE_HEIGHT * maze_config.cell_width + maze_config.wall_width;

        draw.size("100%");

        const px_per_mm = draw.node.clientWidth / maze_width_mm;

        draw.size("100%", maze_height_mm * px_per_mm);


        if (world) {
            world.remove()
            world = undefined;
        }

        world = draw.group();

        world.scale(px_per_mm);

        let maze = world.group();

        self.posts = [];
        self.horizontal_walls = [];
        self.vertical_walls = [];
        for (let i = 0; i < MAZE_WIDTH + 1; i++) {
            self.posts[i] = [];
            self.horizontal_walls[i] = [];
            self.vertical_walls[i] = [];
            for (let j = 0; j < MAZE_HEIGHT + 1; j++) {

                let post = maze.rect(maze_config.wall_width, maze_config.wall_width);
                post.move(i * maze_config.cell_width, j * maze_config.cell_width);
                self.posts[i][j] = post;

                if (i < MAZE_WIDTH) {
                    let wall_color = wall_err_color;

                    if (j === 0 || j === MAZE_WIDTH) {
                        wall_color = wall_closed_color;
                    } else {
                        wall_color = wall_unknown_color;
                    }

                    self.horizontal_walls[i][j] = maze
                        .rect(maze_config.cell_width - maze_config.wall_width, maze_config.wall_width)
                        .move(i * maze_config.cell_width + maze_config.wall_width, j * maze_config.cell_width)
                        .fill(wall_color);
                }

                if (j < MAZE_HEIGHT) {
                    let wall_color = wall_err_color;

                    if (i === 0 || i === MAZE_WIDTH) {
                        wall_color = wall_closed_color;
                    } else {
                        wall_color = wall_unknown_color;
                    }

                    self.vertical_walls[i][j] = maze
                        .rect(maze_config.wall_width, maze_config.cell_width - maze_config.wall_width)
                        .move(i * maze_config.cell_width, j * maze_config.cell_width + maze_config.wall_width)
                        .fill(wall_color);
                }
            }
        }

        let mech = config.mouse.mechanical;

        self.mouse_int = world.group()
        self.mouse_int.rect(mech.length, mech.width).fill(mouse_int_color).translate(mech.front_offset - mech.length, -mech.width / 2);

        self.mouse_ext = world.group()
        self.mouse_ext.rect(mech.length, mech.width).fill(mouse_ext_color).translate(mech.front_offset - mech.length, -mech.width / 2);
    }

    function update(debug) {
        for (let i = 1; i < MAZE_WIDTH; i++) {
            for (let j = 1; j < MAZE_HEIGHT; j++) {
                if (i < MAZE_WIDTH) {
                    let wall = debug.mouse.map.maze.horizontal_edges[i][j - 1];
                    if (wall === "Closed") {
                        self.horizontal_walls[i][j].fill(wall_closed_color)
                    } else if (wall === "Open") {
                        self.horizontal_walls[i][j].fill(wall_open_color)
                    } else if (wall === "Unknown") {
                        self.horizontal_walls[i][j].fill(wall_unknown_color)
                    } else {
                        self.horizontal_walls[i][j].fill(wall_err_color)
                    }
                }

                if (j < MAZE_HEIGHT) {
                    let wall = debug.mouse.map.maze.vertical_edges[i - 1][j];
                    if (wall === "Closed") {
                        self.vertical_walls[i][j].fill(wall_closed_color)
                    } else if (wall === "Open") {
                        self.vertical_walls[i][j].fill(wall_open_color)
                    } else if (wall === "Unknown") {
                        self.vertical_walls[i][j].fill(wall_unknown_color)
                    } else {
                        self.vertical_walls[i][j].fill(wall_err_color)
                    }
                }
            }
        }

        let orientation_int = debug.mouse.orientation;
        self.mouse_int.rotate(orientation_int.direction * 180 / Math.PI).translate(orientation_int.position.x, orientation_int.position.y);

        let orientation_ext = debug.orientation;
        self.mouse_ext.rotate(orientation_ext.direction * 180 / Math.PI).translate(orientation_ext.position.x, orientation_ext.position.y);
    }

    let oldconfig = null;
    let olddebug = null;

    self.update = function (state) {
        if (state.debug()) {
            let debug = state.debug();
            let config = debug.config;
            if (!_.isEqual(config, oldconfig)) {
                console.log("r");
                redraw(config);
                oldconfig = config;
            }
            if (!_.isEqual(debug, olddebug)) {
                update(debug);
                olddebug = debug;
            }
        }
    }
}

function isEqual(a, b) {}


run_worker(initial_config);
