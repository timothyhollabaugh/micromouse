// Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init, { JsSimulation } from './pkg/simulation.js';

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

    px_per_mm: 0.25,

    wall_open_color: '#ffffff',
    wall_closed_color: '#444444',
    wall_unknown_color: '#999999',
    wall_err_color: '#ff0000',

    mouse_int_color: '#00ff00',
    mouse_ext_color: '#ff0000',
};

async function run_simulation(config) {
    await init();

    // And afterwards we can use all the functionality defined in wasm.
    let simulation = new JsSimulation(config.simulation);

    let time_p = document.getElementById("time");

    let root = document.getElementById('ui');

    let ui = new Ui(root, config);

    setInterval(function() {
        let debug = simulation.update(config.simulation);
        time_p.innerText = debug.time;
        ui.update(config, debug);
    }, config.millis_per_step);
}

function Ui(parent, config) {
    let self = this;

    self.root = document.createElement('div');
    parent.append(self.root);

    self.maze_ui = new MazeUi(self.root, config);

    self.update = function(config, debug) {
        self.maze_ui.update(config, debug)

        if (debug.time % 5000 === 0) {
            console.log(debug);
        }
    }
}

function MazeUi(parent, config) {
    self = this;

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

    self.update = function (config, debug) {
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

run_simulation(config);
