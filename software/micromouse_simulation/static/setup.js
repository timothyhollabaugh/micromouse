
function SetupUi(parent, state) {
    let self = this;

    let selected_tab = "simulated";

    let simulated_tab = li();
    let remote_tab = li();

    let simulated = div().children([
        p().text("Default config"),
    ]);

    let remote_url = input().classes('input').style('font-family', 'monospace').value("ws://192.168.4.1:8080");

    let remote = div().children([
        p().text("Default config"),
        fieldset().classes('field has-addons').children([
            div().classes("control").children([
                button().classes("is-static button").text("URL"),
            ]),
            div().classes("control is-expanded").children([remote_url])
        ])
    ]);

    let content = div();

    let connect = a().text("Connect").onclick(function() {
        if (selected_tab === 'simulated') {
            state.connect('simulated', state.simulation_config_default, null);
        } else if (selected_tab === 'remote') {
            state.connect('remote', state.remote_config_default, {url: remote_url.el.value});
        }
    });

    let disconnect = a().text("Disconnect").onclick(function() { state.disconnect(); });

    let root =  card().title("Setup").content([
        div().classes("tabs is-fullwidth").children([
            ul().children([
                simulated_tab.classes("is-active").children([
                    a().text("Simulated").onclick(function() {
                        if (selected_tab === "remote") {
                            remote.el.remove();
                            content.el.append(simulated.el);
                            simulated_tab.classes("is-active");
                            remote_tab.remove_class("is-active");
                            selected_tab = "simulated";
                        }
                    }),
                ]),
                remote_tab.children([
                    a().text("Remote").onclick(function() {
                        if (selected_tab === "simulated") {
                            simulated.el.remove();
                            content.el.append(remote.el);
                            remote_tab.classes("is-active");
                            simulated_tab.remove_class("is-active");
                            selected_tab = "remote";
                        }
                    }),
                ]),
            ]),
        ]),
        content.children([simulated]),
    ]).footer([ connect, disconnect ]);

    parent.append(root.el);
}
