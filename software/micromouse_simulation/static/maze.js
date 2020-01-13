
function MazeUi(parent) {
    let self = this;

    const wall_open_color = '#ffffff';
    const wall_closed_color = '#444444';
    const wall_unknown_color = '#f0f0f0';
    const wall_err_color = '#ff0000';
    const mouse_int_color = '#00ff00';
    const mouse_ext_color = '#ff0000';

    let content = div();

    let root = card().title("Maze").content([content]);

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
            world.remove();
            world = undefined;
        }

        world = draw.group();

        world.scale(px_per_mm, -px_per_mm);
        world.move(maze_config.wall_width/2.0, -maze_height_mm+maze_config.wall_width/2.0);

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
                post.move(i * maze_config.cell_width - maze_config.wall_width/2.0, j * maze_config.cell_width - maze_config.wall_width/2.0);
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
                        .move(i * maze_config.cell_width + maze_config.wall_width/2.0, j * maze_config.cell_width - maze_config.wall_width/2.0)
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
                        .move(i * maze_config.cell_width - maze_config.wall_width/2.0, j * maze_config.cell_width + maze_config.wall_width/2.0)
                        .fill(wall_color);
                }
            }
        }

        let mech = config.mouse.mechanical;

        self.mouse_int = world.group();
        self.mouse_int.rect(mech.length, mech.width).fill(mouse_int_color).translate(mech.front_offset - mech.length, -mech.width / 2);

        self.mouse_target = world.group();
        self.mouse_target.line(0, 0, 100, 0).stroke({color: '#000000', width: 2});

        self.mouse_ext = world.group();
        self.mouse_ext.rect(mech.length, mech.width).fill(mouse_ext_color).translate(mech.front_offset - mech.length, -mech.width / 2);

        self.path = world.path('').fill('none').stroke({color: '#0000ff', width: 2});
        self.path_closest = world.circle(20.0).fill({color: '#0000ff'});
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
        self.mouse_target.rotate(debug.mouse.path.target_direction * 180 / Math.PI).translate(orientation_int.position.x, orientation_int.position.y);

        if (debug.orientation) {
            let orientation_ext = debug.orientation;
            self.mouse_ext.rotate(orientation_ext.direction * 180 / Math.PI).translate(orientation_ext.position.x, orientation_ext.position.y);
        }

        if (debug.mouse.path.path && debug.mouse.path.path.length > 0) {
            let path_string = debug.mouse.path.path.reduce(function(str, segment) {
                let b = segment.bezier;
                return str + "M " + b.start.x + " " + b.start.y + " C " + b.ctrl0.x + " " + b.ctrl0.y + " " + b.ctrl1.x + " " + b.ctrl1.y + " " + b.end.x + " " + b.end.y + " ";
            }, "");

            self.path.plot(path_string);
        } else {
            self.path.plot("");
        }

        if (debug.mouse.path.closest_point) {
            let p = debug.mouse.path.closest_point['1'];
            self.path_closest.translate(p.x - 10.0, p.y - 10.0);
        }
    }

    let oldconfig = null;
    let olddebug = null;

    self.update = function (state) {
        if (state.debug()) {
            let debug = state.debug();
            let config = debug.config;
            if (!_.isEqual(config, oldconfig)) {
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