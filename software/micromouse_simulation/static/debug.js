
function DebugUi(parent) {
    let self = this;

    let content = div();

    let root = card().title("Debug").content([
        content
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
                    self.update(state);
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
                for (let key in nodes) {
                    if (nodes.hasOwnProperty(key)) {
                        if (!data.hasOwnProperty(key)) {
                            nodes[key].root.remove();
                            delete nodes[key];
                        }
                    }
                }
                for (let key in data) {
                    if (data.hasOwnProperty(key)) {
                        if (nodes[key]) {
                            nodes[key].update(state)
                        } else {
                            let node = new Node(path + "/" + key, function(debug) {
                                let value = f(debug);
                                if (value !== null && value.hasOwnProperty(key)) {
                                    return value[key];
                                } else {
                                    return null;
                                }
                            });
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
                    if (isNaN(data)) {
                        value.innerText = 'NaN';
                    } else {
                        value.innerText = math.format(data, {precision: 5, upperExp: 4});
                    }
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
                    state.update();
                }
                if (path in state.graphs) {
                    graphcheck.checked = true;
                }
                header.append(graphcheck);
            }
        }
    };
}
