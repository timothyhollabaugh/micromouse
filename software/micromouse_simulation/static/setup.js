
function SetupUi(parent, state) {
    let self = this;

    let selected_tab = "simulated";

    let simulated_tab = li();
    let remote_tab = li();
    let dump_tab = li();

    let simulated = div().children([
        p().text("Default config"),
    ]);

    let remote_url = input().classes('input').style('font-family', 'monospace').value("ws://localhost:3030");

    let remote = div().children([
        p().text("Default config"),
        fieldset().classes('field has-addons').children([
            div().classes("control").children([
                button().classes("is-static button").text("URL"),
            ]),
            div().classes("control is-expanded").children([remote_url])
        ])
    ]);

    let dump_file_name = span().classes('file-name').text('No file');

    let dump_file = input().type('file').classes('file-input').onchange(function() {
        dump_file_name.text(this.el.files[0].name)
    });

    let dump = div().children([
        div().classes('file has-name').children([
            label().classes('file-label').children([
                dump_file,
                span().classes('file-cta').children([
                    span().classes('file-label').text('Choose a dump file...')
                ])
            ]),
            dump_file_name,
        ])
    ])

    let content = div();

    let connect = a().text("Connect").onclick(function() {
        if (selected_tab === 'simulated') {
            state.connect('simulated', state.simulation_config_default, null);
        } else if (selected_tab === 'remote') {
            state.connect('remote', state.remote_config_default, {url: remote_url.el.value});
        } else if (selected_tab === 'dump') {
            state.connect('dump', state.remote_config_default, {file: dump_file.el.files[0]});
        }
    });

    let disconnect = a().text("Disconnect").onclick(function() { state.disconnect(); });

    let root =  card().title("Setup").content([
        div().classes("tabs is-fullwidth").children([
            ul().children([
                simulated_tab.classes("is-active").children([
                    a().text("Simulated").onclick(function() {
                        if (selected_tab !== "simulated") {
                            remote.el.remove();
                            dump.el.remove();
                            content.el.append(simulated.el);
                            simulated_tab.classes("is-active");
                            remote_tab.remove_class("is-active");
                            dump_tab.remove_class('is-active');
                            selected_tab = "simulated";
                        }
                    }),
                ]),
                remote_tab.children([
                    a().text("Remote").onclick(function() {
                        if (selected_tab !== "remote") {
                            simulated.el.remove();
                            dump.el.remove();
                            content.el.append(remote.el);
                            remote_tab.classes("is-active");
                            simulated_tab.remove_class("is-active");
                            dump_tab.remove_class('is-active');
                            selected_tab = "remote";
                        }
                    }),
                ]),
                dump_tab.children([
                    a().text("Dump").onclick(function() {
                        if (selected_tab !== "dump") {
                            simulated.el.remove();
                            remote.el.remove();
                            content.el.append(dump.el);
                            dump_tab.classes("is-active");
                            simulated_tab.remove_class("is-active");
                            remote_tab.remove_class('is-active');
                            selected_tab = "dump";
                        }
                    })
                ])
            ]),
        ]),
        content.children([simulated]),
    ]).footer([ connect, disconnect ]);

    parent.append(root.el);
}
