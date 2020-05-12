
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
                    if (state.state !== state.STATE_RUNNING && this.el.value >= 0 && this.el.value < state.debugs.length) {
                        state.index = Number(this.el.value);
                        state.update();
                    } else if (this.el.value < 0) {
                        this.value(0);
                    } else if (this.el.value >= state.debugs.length) {
                        this.value(state.debugs.length-1);
                    }
                })
                .onwheel(function(event) {
                    event.preventDefault();
                    if (event.deltaY > 0) {
                        this.el.value = parseInt(this.el.value) - 1;
                    } else if (event.deltaY < 0) {
                        this.el.value = parseInt(this.el.value) + 1;
                    }

                    this.el.oninput();
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
    ]);

    let root = card().title("Control").content([
            div().classes('field is-grouped').children([
                button().classes('control button is-primary').text('Start').style('width', '4em').onclick(function () {
                    if (state.state === state.STATE_RUNNING) {
                        state.stop();
                        this.text('Start');
                        controls.disabled(false);
                    } else {
                        state.start();
                        state.index = -1;
                        this.text('Stop');
                        controls.disabled(true);
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
