
let inited = false;

console.log("webworker!");

importScripts('pkg/simulation.js');

console.log("imported scripts");

async function init() {
    await wasm_bindgen('pkg/simulation_bg.wasm');
    inited = true;
}

init();

console.log("inited");

let simulation = undefined;
let remote = undefined;
let interval_id = undefined;
let websocket = undefined;

postMessage({name: 'start'});

onmessage = function (event) {

    console.log(event.data);

    let msg = event.data;

    if (msg.name === 'connect') {
        if (!simulation) {
            if (msg.data.type === 'simulated') {
                simulation = new wasm_bindgen.JsSimulation(msg.data.config);

                interval_id = setInterval(function() {
                    if (simulation && inited) {
                        let debug = simulation.update();
                        self.postMessage({name: 'debug', data: debug});
                    }
                }, msg.data.config.millis_per_step);

                postMessage({name: 'connected', data: 'simulated'});
            } else if (msg.data.type === 'remote') {
                simulation = new wasm_bindgen.JsRemote(msg.data.config);

                websocket = new WebSocket(msg.data.options.url);

                websocket.onmessage = function(event) {
                    console.log("websocket data: ", event.data);

                    let debug = simulation.update(event.data);
                    self.postMessage({name: 'debug', data: debug});
                };

                websocket.onopen = function(event) {
                    console.log("websocket open");
                    self.postMessage({name: 'connected', data: 'remote'});
                };

                websocket.onclose = function(event) {
                    console.log("websocket closed");
                    self.postMessage({name: 'disconnected'});
                };

                self.postMessage({name: 'connecting', data: 'remote'});
            }
        }
    } else if (msg.name === 'disconnect') {
        if (simulation) {
            simulation = undefined;

            if (interval_id) {
                clearInterval(interval_id);
                interval_id = undefined;
            }

            if (websocket) {
                websocket.close(1000);
                websocket = undefined;
            }
        }
    }
};
