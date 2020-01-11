
function ConfigUi(parent, state) {
    let self = this;

    //let local_config = state.;

    let content = div().classes("content");

    let root = card().title('Config').content([
        content
    ]).footer([
        a().text("Set Config").onclick(function() {
            console.log(local_config);
            state.send_config(local_config);
        })
    ]);

    parent.append(root.el);

    let node = new ConfigNode('config', initial_simulation_config, function(c) {
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

