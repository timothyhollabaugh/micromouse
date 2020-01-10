
function GraphUi(parent, state) {
    let self = this;

    let range = 10000;

    let content = div();

    let root = card().title("Graphs").content([
        div().classes('field has-addons').children([
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
                        state.update();
                    }),
            ]),
            div().classes('control').children([
                button().classes('button is-static').text("ms"),
            ]),
        ]),
        content
    ]);
    parent.append(root.el);

    let oldgraphs = {};

    self.update = function(state) {
        for (let key in state.graphs) {
            if (state.graphs.hasOwnProperty(key)) {
                let f = state.graphs[key];
                if (!(key in oldgraphs)) {
                    oldgraphs[key] = new Graph(content.el, key, state)
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

function Graph(parent, path, state) {
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
                                .oninput(function() {
                                    max = Number(this.el.value);
                                    state.update();
                                }),
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
                                .oninput(function() {
                                    min = Number(this.el.value);
                                    state.update();
                                }),
                        ]),
                    ]),
                ]),
            ]),
        ]),
    ]);
    parent.append(self.root.el);

    let draw = SVG(self.root.el).size("100%", 100);
    let line = draw.polyline([]).fill('none').stroke({width: 1, color: '#444444'});

    let WIDTH = draw.node.clientWidth+2;
    let HEIGHT = draw.node.clientHeight+2;

    let centerline = draw.line(WIDTH/2, 0, WIDTH/2, HEIGHT).stroke({width: 1, color: '#999999'});
    let zeroline = draw.line(0, HEIGHT/2, WIDTH, HEIGHT/2).stroke({width: 1, color: '#999999'});

    self.update = function(range, state, f) {

        let WIDTH = draw.node.clientWidth;
        let HEIGHT = draw.node.clientHeight;

        let points = [];

        let time = state.debug().mouse.time;
        let end_time = state.debugs[state.debugs.length-1].mouse.time;

        let start = time - range;

        if (end_time > range && time > end_time - range/2) {
            start = end_time - range;
        } else if (end_time > range && time > range/2) {
            start = time - range/2;
        }

        if (start < 0) {
            start = 0;
        }

        let debugs = state.debugs
            .map(function(debug, index) { return {time: debug.mouse.time, index: index} })
            .filter(function(debug) { return debug.time > start && debug.time < start + range })

        for (let i = 0; i < debugs.length; i++) {
            let index = debugs[i].index;
            let time = debugs[i].time;
            let value = f(state, index) - min;
            points[i] = [(time - start) * WIDTH / range, HEIGHT - value * HEIGHT / (max - min)+1];
        }

        //line.clear();
        line.plot(points);

        let center = time - start;
        centerline.plot(center * WIDTH/range, 0, center * WIDTH/range, HEIGHT);

        let zero = -min * HEIGHT / (max - min) + 1;

        if (min > 0 && max > 0) {
            zero = 0;
        }

        if (min < 0 && max < 0) {
            zero = HEIGHT;
        }

        zeroline.plot(0, HEIGHT-zero, WIDTH, HEIGHT-zero);
    }
}
