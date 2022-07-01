use byteorder::ReadBytesExt;
use flume::{Receiver, Sender};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{memory, JsCast};
use web_sys::{
    console, DedicatedWorkerGlobalScope, MessageEvent, Worker, WorkerOptions, WorkerType,
};

static RECORDED_I8: &[u8] = include_bytes!("../../record.raw");

lazy_static! {
    static ref CHANNEL: (Sender<u8>, Receiver<u8>) = flume::bounded::<u8>(32);
}

#[wasm_bindgen]
pub struct Data {
    cursor: Cursor<&'static [u8]>,
}

#[wasm_bindgen]
impl Data {
    pub fn new() -> Data {
        Data {
            cursor: Cursor::new(RECORDED_I8),
        }
    }

    pub fn read_n(&mut self, n: u32) -> Vec<i8> {
        let mut v = Vec::with_capacity(2048);
        for _ in 0..n {
            if self.cursor.position() >= (RECORDED_I8.len() - 1) as u64 {
                self.cursor.set_position(0);
            }
            v.push(self.cursor.read_i8().unwrap());
        }
        v
    }
}

/// A number evaluation struct
///
/// This struct will be the main object which responds to messages passed to the
/// worker. It stores the last number which it was passed to have a state. The
/// statefulness is not is not required in this example but should show how
/// larger, more complex scenarios with statefulness can be set up.
#[wasm_bindgen]
#[derive(Default)]
pub struct NumberEval {
    number: i32,
}

#[wasm_bindgen]
impl NumberEval {
    /// Create new instance.
    pub fn new() -> NumberEval {
        NumberEval { number: 0 }
    }

    /// Check if a number is even and store it as last processed number.
    ///
    /// # Arguments
    ///
    /// * `number` - The number to be checked for being even/odd.
    pub fn is_even(&mut self, number: i32) -> bool {
        self.number = number;
        matches!(self.number % 2, 0)
    }

    /// Get last number that was checked - this method is added to work with
    /// statefulness.
    pub fn get_last_number(&self) -> i32 {
        self.number
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SendMe {
    s: String,
    data: Vec<i8>,
}

impl Default for SendMe {
    fn default() -> Self {
        Self {
            s: "SomeString".to_owned(),
            data: vec![10; 10],
        }
    }
}

#[wasm_bindgen]
pub async fn send_stuff() {
    let global = js_sys::global();
    let global = global.dyn_into::<DedicatedWorkerGlobalScope>().unwrap();
    loop {
        // check a state variable which can be changed via message callback from the main/UI thread
        // We don't need to send the data to the main/UI thread, send it to the server instead
        // Terminate and instantiate new worker with different decoder (or whatever)
        // In worker, use extern "get_frames" to get frames from the driver
        // What to do in main/UI thread? Create a status/info site and update via messages from
        // controller and decoder workers.
        // controller uses websockets to get new commands (store and retrieve or push to node?)
        // worker has fixed workload and executes it, sends data to backend (init method to prevent hard coding?)
        gloo_timers::future::sleep(std::time::Duration::from_secs(1)).await;
        console::log_1(&"send_stuff".into());
        let mut x = SendMe::default();
        //x.data = read_samples();
        global
            .post_message(&JsValue::from_serde(&x).unwrap())
            .unwrap();
        console::log_1(&"trying to receive".into());
        while let Ok(x) = CHANNEL.1.recv_async().await {
            console::log_1(&format!("{:?}", x).into());
        }
    }
}

/// Run entry point for the main thread.
#[wasm_bindgen]
pub async fn startup() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    actually_do_stuff().await;
}

pub async fn actually_do_stuff() {
    let mut options = WorkerOptions::new();
    options.type_(WorkerType::Module);
    let worker_handle = Worker::new_with_options("./new_worker.js", &options).unwrap();
    // let worker_handle = Worker::new("./new_worker.js").unwrap();
    let callback = log_callback();
    worker_handle.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
    //worker_handle.post_message(&memory()).unwrap();
    loop {
        console::log_1(&"channel_send".into());
        CHANNEL.0.send_async(0x10_u8).await.unwrap();
    }
}

/// Create a closure to act on the message returned by the worker
fn log_callback() -> Closure<dyn FnMut(MessageEvent)> {
    Closure::wrap(Box::new(move |event: MessageEvent| {
        let x: SendMe = event.data().into_serde().unwrap();
        let msg = format!("Incoming message: {:?}", x);
        console::log_1(&msg.into());
    }) as Box<dyn FnMut(_)>)
}
