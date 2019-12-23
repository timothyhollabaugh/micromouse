
let inited = false;

console.log("webworker!");

importScripts('pkg/simulation.js');

console.log("imported scripts");

async function init() {
    await wasm_bindgen('pkg/simulation_bg.wasm');
}

init();

console.log("inited");
inited = true;

let simulation = null;
let config = null;
let interval_id = null;

self.onmessage = function (event) {
    if (event.data.name === 'config') {
        config = event.data.data;
    } else if (event.data.name === 'start') {
        if (config && !interval_id) {
            interval_id = setInterval(function() {
                if (!simulation && inited && config !== null) {
                    simulation = new wasm_bindgen.JsSimulation(config);
                }
                if (simulation && inited) {
                    let debug = simulation.update(config);
                    self.postMessage(debug);
                }
            }, config.millis_per_step)
        }
    } else if (event.data.name === 'stop') {
        if (interval_id) {
            clearInterval(interval_id);
            interval_id = null;
        }
    }
};
