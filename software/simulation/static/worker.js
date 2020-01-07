
let inited = false;

console.log("webworker!");

importScripts('pkg/simulation.js');

console.log("imported scripts");

function Simulation(config, send) {
    let self = this;

    let simulation = new wasm_bindgen.JsSimulation(config);

    let interval_id = undefined;

    send({name: 'connected'});

    self.start = function() {
        if (!interval_id) {
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

    self.config = function(config) {
        simulation.config(config);
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
    };

    let socket = new WebSocket(url);
    socket.binaryType = 'arraybuffer';

    socket.onopen = function(event) {
        console.log("websocket open");
        send_byte(BYTE_START_DEBUG);
        state = STATE_OK;
        send({name: 'connected'});
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

    self.config = function () {};
}

async function init() {
    await wasm_bindgen('pkg/simulation_bg.wasm');

    console.log("start");

    wasm_bindgen.init_wasm();

    postMessage({name: 'loaded'});

    let handler = undefined;

    onmessage = function (event) {
        let msg = event.data;

        if (msg.name === 'connect') {
            if (!handler) {
                if (msg.data.type === 'simulated') {
                    handler = new Simulation(msg.data.config, function(m) { postMessage(m) });
                } else if (msg.data.type === 'remote') {
                    handler = new Remote(msg.data.config, msg.data.options.url, function(m) { postMessage(m) });
                }
                postMessage({name: 'connecting'});
            }
        } else if (msg.name === 'disconnect') {
            handler.stop();
            handler.disconnect();
            handler = undefined;
            postMessage({name: 'disconnected'});
        } else if (msg.name === 'start') {
            handler.start();
            postMessage({name: 'running'});
        } else if (msg.name === 'stop') {
            handler.stop();
            postMessage({name: 'stopped'});
        } else if (msg.name === 'config') {
            handler.config(msg.data);
        }
    };
}

init();
