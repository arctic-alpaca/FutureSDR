// The worker has its own scope and no direct access to functions/objects of the
// global scope. We import the generated JS file to make `wasm_bindgen`
// available which we need to initialize our WASM code.
//
// importScripts('./node.js');

console.log('Initializing worker')

// In the worker, we have a different struct that we want to use as in
// `index.js`.
import init, {send_stuff, NumberEval} from "./node.js";

async function init_wasm_in_worker() {

    onmessage = async function (m) {
        if (m.data.task === "init") {
            await init(undefined, m.data.mem);
            id = m.data.id;
            console.log("Worker", id, "init done");
            postMessage("init done");
            send_stuff();
        }
    }
    /*
    // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`.
    await wasm_bindgen('./node_bg.wasm');

    send_stuff();

    // Create a new object of the `NumberEval` struct.
    var num_eval = NumberEval.new();

    // Set callback to handle messages passed to the worker.
    self.onmessage = async event => {
        // By using methods of a struct as reaction to messages passed to the
        // worker, we can preserve our state between messages.
        var worker_result = num_eval.is_even(event.data);


        // Send response back to be handled by callback in main thread.
        self.postMessage(worker_result);
    };*/
};

init_wasm_in_worker();


