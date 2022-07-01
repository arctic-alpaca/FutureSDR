
import init, {send_stuff} from "./node.js";

async function init_new_wasm_in_worker() {

    await init();

    await send_stuff();

}

console.log('Initializing new_worker')

await init_new_wasm_in_worker();