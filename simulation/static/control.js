
function ControlUi(parent, state) {
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
                    if (!state.state === state.STATE_RUNNING && this.el.value > 0 && this.el.value < state.debugs.length) {
                        state.index = Number(this.el.value);
                        state.update();
                    }
                })
                .onupdate(function(state) {
                    if (state.state === state.STATE_RUNNING) {
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
    ]).onupdate(function(state) {
        if (state.state === state.STATE_STOPPED) {
            this.disabled(false);
        } else {
            this.disabled(true);
        }
    });

    let root = card().title("Control").content([
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
    ]);

    parent.append(root.el);

    self.update = function(state) {
        root.update(state);
    }
}
