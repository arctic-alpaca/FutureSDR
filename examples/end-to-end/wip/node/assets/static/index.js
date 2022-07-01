
import init, {startup} from "./node.js";

async function run_wasm() {

    const mod = await init();

    const mem = mod.memory;
    if (!(mem instanceof WebAssembly.Memory)) {
        console.log("mod", mod);
        throw new Error("Maybe exports have changed? Try renaming __wbindgen_export_0 to memory");
    }
    else {
        console.log("memory done");
    }

    async function createWorkers(mem, count) {
        const workers = [...Array(count).keys()].map(
            (id) =>
                new Promise((res, _rej) => {
                    const rejectedTimeout = setTimeout(() => rej("timeout"), 3000);

                    // Create worker
                    const worker = new Worker("./worker.js", {
                        type: "module",
                    });

                    // Send the shared memory to worker
                    worker.postMessage({ task: "init", mem, id });

                    // Wait for the worker to report back
                    worker.onmessage = function (e) {
                        worker.onmessage = undefined;
                        if (e.data === "init done") {
                            clearTimeout(rejectedTimeout);
                            res(worker);
                        }
                    };
                })
        );

        return await Promise.all(workers);
    }

    const workers = await createWorkers(mem, 4);

    console.log('index.js loaded');

    // Run main WASM entry point
    // This will create a worker from within our Rust code compiled to WASM
    await startup();
}

run_wasm();

