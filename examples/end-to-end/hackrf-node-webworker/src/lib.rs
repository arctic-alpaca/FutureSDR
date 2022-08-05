use fft_worker::FftWorker;
use futuresdr::log;
use gloo_worker::Spawnable;
use gloo_worker::WorkerBridge;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static FFT_WORKER: RefCell<Option<WorkerBridge<FftWorker>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn input_samples(samples: Vec<i8>) {
    FFT_WORKER.with(|bridge| {
        if let Some(bridge) = bridge.borrow_mut().as_ref() {
            bridge.send(samples.clone());
        }
    });
    // Add more potential workers here
}

#[wasm_bindgen]
pub async fn create_fft_worker() {
    let bridge = FftWorker::spawner()
        //.encoding::<MyCodec>()
        .spawn("fft_worker.js");
    FFT_WORKER.with(|x| {
        x.replace(Some(bridge));
    });
}

#[wasm_bindgen]
pub fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[wasm_bindgen]
pub async fn setup_logger() {
    console_log::init_with_level(log::Level::Trace).unwrap();
}
