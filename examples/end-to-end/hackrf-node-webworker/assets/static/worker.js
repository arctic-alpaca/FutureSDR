import init, {run_worker} from "./hackrf_node_webworker.js"

async function run_in_worker() {
    await init();
    console.log("Worker JS file loaded");

    run_worker();
}

await run_in_worker();
