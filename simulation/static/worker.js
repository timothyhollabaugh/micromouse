
let inited = false;

console.log("webworker!");

importScripts('pkg/simulation.js');

console.log("imported scripts");

function Simulation(config, send) {
    let self = this;

    let simulation = new wasm_bindgen.JsSimulation(config);

    let interval_id = undefined;

    self.start = function() {
        if (!interval_id) {
            console.log(config.millis_per_step);
            interval_id = setInterval(function() {
                let debug = simulation.update();
                send({
                    name: 'debug',
                    data: debug,
                });
            }, config.millis_per_step);
        }
    };

    self.stop = function() {
        if (interval_id) {
            clearInterval(interval_id);
            interval_id = undefined;
        }
    };

    self.disconnect = function() { }
}

function Remote(config, url, send) {
    let self = this;

    let send_byte = function(b) {
        let data = new Uint8Array(1);
        data[0] = b;
        socket.send(data);
    };

    const BYTE_STOP_DEBUG = 1;
    const BYTE_START_DEBUG = 2;
    const BYTE_STOP = 3;
    const BYTE_START = 4;

    const STATE_CONNECTING = 'connecting';
    const STATE_OK = 'ok';
    const STATE_ERR = 'err';
    const STATE_CLOSED = 'closed';

    const ERR_TIMEOUT = 100;

    let state = STATE_CONNECTING;

    let remote = new wasm_bindgen.JsRemote(config);

    let err_timeout_id = undefined;
    let err_timeout_fn = function() {
        console.log("err timed out");
        send_byte(BYTE_START_DEBUG);
        state = STATE_OK;
    }

    let socket = new WebSocket(url);
    socket.binaryType = 'arraybuffer';

    socket.onopen = function(event) {
        console.log("websocket open");
        send_byte(BYTE_START_DEBUG);
        state = STATE_OK;
    };

    socket.onclose = function(event) {
        console.log("websocket closed");
        state = STATE_CLOSED;
    };

    socket.onmessage = function(event) {
        if (state === STATE_OK) {
            let data = new Uint8Array(event.data);
            let result = remote.update(data);
            if ("Ok" in result) {
                let debugs = result["Ok"];
                debugs.forEach(function(debug) {
                    send({name: 'debug', data: debug});
                });
            } else if ("Err" in result) {
                console.log("comms error");
                send_byte(BYTE_STOP_DEBUG);
                err_timeout_id = setTimeout(err_timeout_fn, ERR_TIMEOUT);
                state = STATE_ERR;
            }
        } else if (state === STATE_ERR) {
            console.log("bytes in error");
            if (err_timeout_id) {
                clearTimeout(err_timeout_id);
            }

            err_timeout_id = setTimeout(err_timeout_fn, ERR_TIMEOUT);
        }
    };

    self.start = function() {
        send_byte(BYTE_START);
    };

    self.stop = function() {
        send_byte(BYTE_STOP);
    };

    self.disconnect = function() {
        socket.close();
    }
}

async function init() {
    await wasm_bindgen('pkg/simulation_bg.wasm');

    console.log("start");

    postMessage({name: 'start'});

    let handler = undefined;

    onmessage = function (event) {
        console.log(event.data);

        let msg = event.data;

        if (msg.name === 'connect') {
            if (!handler) {
                if (msg.data.type === 'simulated') {
                    handler = new Simulation(msg.data.config, function(m) { postMessage(m) });
                } else if (msg.data.type === 'remote') {
                    handler = new Remote(msg.data.config, msg.data.options.url, function(m) { postMessage(m) });
                }
            }
        } else if (msg.name === 'disconnect') {
            handler.disconnect();
            handler = undefined;
        } else if (msg.name === 'start') {
            handler.start();
        } else if (msg.name === 'stop') {
            handler.stop();
        }
    };
}

init();

    /*

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
                websocket.binaryType = 'arraybuffer';

                websocket.onmessage = function(event) {
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
    } else if (msg.name === 'start') {
        if (simulation) {
            if (websocket) {
                let data = new Uint8Array(1);
                data[0] = 4;
                websocket.send(data);
            }
        }
    } else if (msg.name === 'stop') {
        if (simulation) {
            if (websocket) {
                let data = new Uint8Array(1);
                data[0] = 3;
                websocket.send(data);
            }
        }
    }
     */
