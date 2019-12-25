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

const config = {
    simulation: {
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
    },

    px_per_mm: 0.2,

    wall_open_color: '#ffffff',
    wall_closed_color: '#444444',
    wall_unknown_color: '#999999',
    wall_err_color: '#ff0000',

    mouse_int_color: '#00ff00',
    mouse_ext_color: '#ff0000',
};

function UiState(send) {
    let self = this;

    self.config = config;
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
}

function run_worker(config) {
    if (window.Worker) {
        let worker = new Worker('worker.js');
        worker.postMessage({name: 'config', data: config.simulation});

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

    self.root = document.createElement('div');
    self.root.className = 'columns container';
    parent.append(self.root);


    self.debug_div = document.createElement('div');
    self.debug_div.className = 'column is-narrow';
    self.debug_div.style.width = '25em';
    self.root.append(self.debug_div);

    self.simulation_ui = new SimulationUi(self.debug_div, state);

    self.maze_div = document.createElement('div');
    self.maze_div.className = 'column is-narrow';
    self.root.append(self.maze_div);
    self.maze_ui = new MazeUi(self.maze_div, state);
    self.graph_ui = new GraphUi(self.maze_div, state);

    self.debug_ui = new DebugUi(self.debug_div, state);

    self.update = function(state) {
        self.simulation_ui.update(state);
        self.maze_ui.update(state);
        self.graph_ui.update(state);
        self.debug_ui.update(state);
    }
}

function SimulationUi(parent, state) {
    let self = this;

    let controls = fieldset().classes('control field has-addons').disabled(true).children([
        p().classes('control').children([
            input('number')
                .classes('input')
                .style('text-align', 'right')
                .style('fontFamily', 'monospace')
                .style('width', '6em')
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
        p().classes('control').children([
            button()
                .classes('button is-static')
                .text('/ 0')
                .style('fontFamily', 'monospace')
                .onupdate(function(state) {
                    this.text('/ ' + (state.debugs.length-1))
                })
        ])
    ]);

    let root = div().classes('field is-grouped').children([
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
    ]);

    parent.append(root.el);

    self.update = function(state) {
        root.update(state);
    }
}

function p() {
    return new El('p');
}

function div() {
    return new El('div');
}

function button() {
    return new El('button');
}

function fieldset() {
    return new El('fieldset');
}

function input(type) {
    let input = new El('input');
    input.el.type = type;
    input.value = function(value) {
        input.el.value = value;
    };
    input.min = function(min) {
        input.el.min = min;
    };
    input.max = function(max) {
        input.el.max = max;
    };
    return input;
}

function El(tag) {
    let self = this;

    self.el = document.createElement(tag);

    let children = [];
    let update = undefined;

    self.classes = function(classes) {
        self.el.className += classes;
        return self;
    };

    self.text = function(text) {
        self.el.innerText = text;
        return self;
    };

    self.children = function(c) {
        for (let i = 0; i < c.length; i++) {
            self.el.append(c[i].el);
            children.push(c[i]);
        }
        return self;
    };

    self.onclick = function(f) {
        self.el.onclick = f.bind(self);
        return self;
    };

    self.oninput = function(f) {
        self.el.oninput = f.bind(self);
        return self;
    }

    self.disabled = function(d) {
        self.el.disabled = d;
        return self;
    };

    self.style = function(s, v) {
        self.el.style[s] = v;
        return self;
    }

    self.onupdate = function(f) {
        update = f.bind(self);
        return self;
    };

    self.update = function(state) {
        if (update) {
            update(state);
        }

        for (let i = 0; i < children.length; i++) {
            children[i].update(state);
        }
    };
}

function DebugUi(parent, config) {
    let self = this;

    self.root = document.createElement('div');
    parent.append(self.root);

    self.node = new Node('debug');
    self.root.append(self.node.root);

    self.update = function(state) {
        self.node.update(state.debug());
    }
}

function Node(key) {
    let self = this;

    self.root = document.createElement('div');

    let header = document.createElement('div');
    self.root.append(header);

    let name = document.createElement('span');
    name.innerText = key;
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

    self.update = function(data) {
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
                            nodes[key].update(data[key])
                        } else {
                            let node = new Node(key);
                            node.update(data[key]);
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
        } else {
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
        }
    };
}

function GraphUi(parent, state) {
    let self = this;

    let root = div();
    parent.append(root.el);

    let graph = new Graph(root.el);

    self.update = function(state) {
        graph.update(1000, 1000, 2000, state, function(state, index) {
            return state.debugs[index].mouse_debug.orientation.position.x;
        })
    }

}

function Graph(parent) {
    let self = this;

    const WIDTH = 1000;
    const HEIGHT = 200;

    let draw = SVG(parent).size(WIDTH, HEIGHT);
    let line = draw.polyline([]).fill('none').stroke({width: 2});

    let centerline = draw.line(WIDTH/2, 0, WIDTH/2, HEIGHT).stroke({width: 1, color: '#999999'});
    let zeroline = draw.line(0, HEIGHT/2, WIDTH, HEIGHT/2).stroke({width: 1, color: '#999999'});

    self.update = function(range, min, max, state, f) {
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

function MazeUi(parent, state) {
    let self = this;

    const config = state.config;
    const maze_config = config.simulation.mouse.map.maze;
    const maze_width_mm = MAZE_WIDTH * maze_config.cell_width + maze_config.wall_width;
    const maze_height_mm = MAZE_HEIGHT * maze_config.cell_width + maze_config.wall_width;

    let draw = SVG(parent).size(maze_width_mm * config.px_per_mm, maze_height_mm * config.px_per_mm);

    let world = draw.group();

    world.scale(config.px_per_mm);

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
                let wall_color = config.wall_err_color;

                if (j === 0 || j === MAZE_WIDTH) {
                    wall_color = config.wall_closed_color;
                } else {
                    wall_color = config.wall_unknown_color;
                }

                self.horizontal_walls[i][j] = maze
                    .rect(maze_config.cell_width - maze_config.wall_width, maze_config.wall_width)
                    .move(i * maze_config.cell_width + maze_config.wall_width, j * maze_config.cell_width)
                    .fill(wall_color);
            }

            if (j < MAZE_HEIGHT) {
                let wall_color = config.wall_err_color;

                if (i === 0 || i === MAZE_WIDTH) {
                    wall_color = config.wall_closed_color;
                } else {
                    wall_color = config.wall_unknown_color;
                }

                self.vertical_walls[i][j] = maze
                    .rect(maze_config.wall_width, maze_config.cell_width - maze_config.wall_width)
                    .move(i * maze_config.cell_width, j * maze_config.cell_width + maze_config.wall_width)
                    .fill(wall_color);
            }
        }
    }

    let mech = config.simulation.mouse.mechanical;

    self.mouse_int = world.group()
    self.mouse_int.rect(mech.length, mech.width).fill(config.mouse_int_color).translate(mech.front_offset - mech.length, -mech.width/2);

    self.mouse_ext = world.group()
    self.mouse_ext.rect(mech.length, mech.width).fill(config.mouse_ext_color).translate(mech.front_offset - mech.length, -mech.width/2);

    self.update = function (state) {
        if (state.debug()) {
            let config = state.config;
            let debug = state.debug();
            for (let i = 1; i < MAZE_WIDTH; i++) {
                for (let j = 1; j < MAZE_HEIGHT; j++) {
                    if (i < MAZE_WIDTH) {
                        let wall = debug.mouse_debug.map.maze.horizontal_edges[i][j - 1];
                        if (wall === "Closed") {
                            self.horizontal_walls[i][j].fill(config.wall_closed_color)
                        } else if (wall === "Open") {
                            self.horizontal_walls[i][j].fill(config.wall_open_color)
                        } else if (wall === "Unknown") {
                            self.horizontal_walls[i][j].fill(config.wall_unknown_color)
                        } else {
                            self.horizontal_walls[i][j].fill(config.wall_err_color)
                        }
                    }

                    if (j < MAZE_HEIGHT) {
                        let wall = debug.mouse_debug.map.maze.vertical_edges[i - 1][j];
                        if (wall === "Closed") {
                            self.vertical_walls[i][j].fill(config.wall_closed_color)
                        } else if (wall === "Open") {
                            self.vertical_walls[i][j].fill(config.wall_open_color)
                        } else if (wall === "Unknown") {
                            self.vertical_walls[i][j].fill(config.wall_unknown_color)
                        } else {
                            self.vertical_walls[i][j].fill(config.wall_err_color)
                        }
                    }
                }
            }

            let orientation_int = debug.mouse_debug.orientation;
            self.mouse_int.rotate(orientation_int.direction * 180 / Math.PI).translate(orientation_int.position.x, orientation_int.position.y);

            let orientation_ext = debug.orientation;
            self.mouse_ext.rotate(orientation_ext.direction * 180 / Math.PI).translate(orientation_ext.position.x, orientation_ext.position.y);
        }
    }
}

function run_render3d() {

    const SCREEN_WIDTH = 640
    const SCREEN_HEIGHT = 480
    const aspect = SCREEN_WIDTH / SCREEN_HEIGHT;
    const frustumSize = 6000;

    var renderer = new THREE.WebGLRenderer();
    renderer.setSize(SCREEN_WIDTH, SCREEN_HEIGHT);
    document.body.appendChild( renderer.domElement );

    let scene = new THREE.Scene();
    let camera = new THREE.PerspectiveCamera(
        75,
        aspect,
        0.1,
        10000,
    )
    /*
    let camera = new THREE.OrthographicCamera(
        0.5 * frustumSize * aspect / - 2,
        0.5 * frustumSize * aspect / 2,
        frustumSize / 2,
        frustumSize / - 2,
        0,
        10000
    );
    */

    let controls = new THREE.OrbitControls(camera, renderer.domElement);

    const maze_config = config.simulation.mouse.map.maze;
    const maze_width_mm = MAZE_WIDTH * maze_config.cell_width + maze_config.wall_width;
    const maze_height_mm = MAZE_HEIGHT * maze_config.cell_width + maze_config.wall_width;

    let base_material = new THREE.MeshLambertMaterial( { color: 0xdddddd } );
    let base_geometery = new THREE.BoxGeometry(maze_width_mm, 20.0, maze_height_mm);
    let base_mesh = new THREE.Mesh(base_geometery, base_material);
    base_mesh.position.x = maze_width_mm/2.0;
    base_mesh.position.y = -10.0
    base_mesh.position.z = -(maze_height_mm/2.0);
    scene.add(base_mesh);

    let post_material = new THREE.MeshLambertMaterial( { color: 0x222222 } );
    let wall_unknown_material = new THREE.MeshLambertMaterial( { color: 0xcccccc, transparent: true, opacity: 0.5 } );
    let wall_open_material = new THREE.MeshLambertMaterial( { color: 0xcccccc, transparent: true, opacity: 0.0 } );
    let wall_closed_material = new THREE.MeshLambertMaterial( { color: 0xcccccc, transparent: false});
    let wall_err_material = new THREE.MeshLambertMaterial( { color: 0xff0000 } );

    let posts = [];
    let horizontal_walls = [];
    let vertical_walls = [];
    for (let i = 0; i < MAZE_WIDTH + 1; i++) {
        posts[i] = [];
        horizontal_walls[i] = [];
        vertical_walls[i] = [];
        for (let j = 0; j < MAZE_HEIGHT + 1; j++) {

            let post_geometry = new THREE.BoxGeometry(maze_config.wall_width, config.wall_height, maze_config.wall_width);
            let post_mesh = new THREE.Mesh(post_geometry, post_material);
            post_mesh.position.x = i * maze_config.cell_width + maze_config.wall_width/2.0;
            post_mesh.position.y = config.wall_height/2.0;
            post_mesh.position.z = -(j * maze_config.cell_width + maze_config.wall_width/2.0);
            scene.add(post_mesh);
            posts[i][j] = post_mesh;

            if (i < MAZE_WIDTH) {
                let wall_material = wall_err_material;

                if (i === 0 || i === MAZE_WIDTH-1) {
                    wall_material = wall_closed_material;
                } else {
                    wall_material = wall_unknown_material;
                }

                let wall_geometry = new THREE.BoxGeometry(maze_config.cell_width - maze_config.wall_width, config.wall_height, maze_config.wall_width);
                let wall_mesh = new THREE.Mesh(wall_geometry, wall_material);
                wall_mesh.position.x = i * maze_config.cell_width + maze_config.wall_width + (maze_config.cell_width - maze_config.wall_width)/2.0;
                wall_mesh.position.y = config.wall_height/2.0
                wall_mesh.position.z = -(j * maze_config.cell_width + maze_config.wall_width/2.0);
                scene.add(wall_mesh);
                horizontal_walls[i][j] = wall_mesh;
            }

            if (j < MAZE_HEIGHT) {
                let wall_material = wall_err_material;

                //if (j === 0 || j === MAZE_WIDTH-1) {
                    //wall_material = wall_closed_material;
                //} else {
                    //wall_material = wall_unknown_material;
                //}

                let wall_geometry = new THREE.BoxGeometry(maze_config.wall_width, config.wall_height, maze_config.cell_width - maze_config.wall_width);
                let wall_mesh = new THREE.Mesh(wall_geometry, wall_material);
                wall_mesh.position.x = i * maze_config.cell_width + maze_config.wall_width/2.0;
                wall_mesh.position.y = config.wall_height/2.0
                wall_mesh.position.z = -(j * maze_config.cell_width + maze_config.wall_width + (maze_config.cell_width - maze_config.wall_width)/2.0);
                scene.add(wall_mesh);
                vertical_walls[i][j] = wall_mesh;
            }
        }
    }

    let lights = [];
    for (let i = 0; i < MAZE_WIDTH; i++) {
        lights[i] = [];
        for (let j = 0; j < MAZE_HEIGHT; j++) {
            let light = new THREE.PointLight(0xffffff, 1.0, 0.0);
            light.position.x = i * maze_config.cell_width + maze_config.cell_width/2;
            light.position.y = config.wall_height * 2.0;
            light.position.z = j * maze_config.cell_width + maze_config.cell_width/2;
            scene.add(light);

            lights[i][j] = light;
        }
    }

    camera.position.z = 200;
    controls.update();

    let animate = function () {
        requestAnimationFrame( animate );

        if (debug && debug.mouse_debug) {
            for (let i = 1; i < MAZE_WIDTH; i++) {
                for (let j = 1; j < MAZE_HEIGHT; j++) {
                    if (i < MAZE_WIDTH - 1) {
                        let edge = debug.mouse_debug.map.maze.horizontal_edges[i][j];

                        let wall_material = wall_err_material;
                        if (edge === "Closed") {
                            wall_material = wall_closed_material;
                        } else if (edge === "Open") {
                            wall_material = wall_open_material;
                        } else if (edge === "Unknown") {
                            wall_material = wall_unknown_material;
                        }

                        horizontal_walls[i][j].material = wall_material;
                    }
                }
            }
        }

        controls.update();
        renderer.render( scene, camera );
    };

    animate();
}

//run_simulation(config);
run_worker(config);
