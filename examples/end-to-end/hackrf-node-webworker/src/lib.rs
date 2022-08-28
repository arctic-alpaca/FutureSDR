use control_worker::ControlWorker;
use fft_worker::FftWorker;
use gloo_worker::Spawnable;
use gloo_worker::WorkerBridge;
use shared_utils::{DataTypeMarker, FromCommandMsg, NodeConfig, ToCommandMsg, ToWorker};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

thread_local! {
    static CONTROL_WORKER: RefCell<Option<WorkerBridge<ControlWorker>>> = RefCell::new(None);
    static FFT_WORKER: RefCell<Option<WorkerBridge<FftWorker>>> = RefCell::new(None);
    //static ZIG_BEE_WORKER: RefCell<Option<WorkerBridge<ZigBeeWorker>>> = RefCell::new(None);

    // Add globals for more workers here
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn print_from_rust(s: &str);
    #[wasm_bindgen]
    fn set_freq_from_rust(freq: u64);
    #[wasm_bindgen]
    fn set_amp_from_rust(amp: u8);
    #[wasm_bindgen]
    fn set_lna_from_rust(lna: u8);
    #[wasm_bindgen]
    fn set_vga_from_rust(vga: u8);
    #[wasm_bindgen]
    fn set_sample_rate_from_rust(sample_rate: u64);
}

#[wasm_bindgen]
pub fn input_samples(samples: Vec<i8>) {
    FFT_WORKER.with(|bridge| {
        if let Some(bridge) = bridge.borrow_mut().as_ref() {
            bridge.send(ToWorker::Data {
                data: samples.clone(),
            });
        }
    });
    // Add data input for more workers here
}

pub fn stop_data_workers() {
    // Replacing the FFT_WORKER content drops the WorkerBridge with terminates the worker.
    FFT_WORKER.with(|inner| {
        inner.replace(None);
    });
    // Add stop code for more workers here
}

pub fn stop_control_workers() {
    CONTROL_WORKER.with(|inner| {
        inner.replace(None);
    });
}

pub fn start_data_workers(node_config: &NodeConfig) {
    for entry in &node_config.data_types {
        match entry {
            DataTypeMarker::Fft => {
                FFT_WORKER.with(|inner| {
                    inner.replace(Some(FftWorker::spawner().spawn("fft_worker.js")));
                    if let Some(bridge) = &*inner.borrow_mut() {
                        bridge.send(ToWorker::ApplyConfig {
                            config: node_config.clone(),
                        });
                    }
                });
            }
            DataTypeMarker::ZigBee => {}
        }
    }
    // Add start code for more workers here
}

pub fn apply_config(node_config: NodeConfig) {
    stop_data_workers();

    set_freq_from_rust(node_config.freq);
    set_amp_from_rust(node_config.amp);
    set_lna_from_rust(node_config.lna);
    set_vga_from_rust(node_config.vga);
    set_sample_rate_from_rust(node_config.sample_rate);

    start_data_workers(&node_config);
}

pub fn restart_control_worker() {
    create_control_worker();
    CONTROL_WORKER.with(|inner| {
        if let Some(bridge) = &*inner.borrow_mut() {
            bridge.send(ToCommandMsg::GetInitialConfig);
        }
    });
    initialize_config();
}

#[wasm_bindgen]
pub fn create_control_worker() {
    let bridge = ControlWorker::spawner()
        .callback(move |m| match m {
            FromCommandMsg::ReceivedConfig { config } => {
                print_from_rust(&format!("Applying new config: {config:?}"));
                apply_config(config.clone());

                CONTROL_WORKER.with(|inner| {
                    let bridge_option = inner.borrow();
                    let bridge = bridge_option.as_ref().unwrap();
                    bridge.send(ToCommandMsg::AckConfig { config });
                })
            }
            FromCommandMsg::PrintToScreen { msg } => {
                print_from_rust(&msg);
            }
            FromCommandMsg::Terminate { msg } => {
                print_from_rust(&msg);
                stop_data_workers();
                stop_control_workers();
            }
            FromCommandMsg::Disconnected => {
                stop_control_workers();
                stop_data_workers();
                print_from_rust(
                    "No connection to backend, restarting workers after 10 seconds to reconnect",
                );
                spawn_local(async {
                    gloo_timers::future::TimeoutFuture::new(10_000).await;
                    restart_control_worker();
                });
            }
        })
        .spawn("control_worker.js");

    CONTROL_WORKER.with(|inner| {
        inner.replace(Some(bridge));
    });
}

#[wasm_bindgen]
pub fn initialize_config() {
    CONTROL_WORKER.with(|inner| {
        if let Some(bridge) = &*inner.borrow_mut() {
            bridge.send(ToCommandMsg::Initialize);
            bridge.send(ToCommandMsg::GetInitialConfig);
        }
    });
}

#[wasm_bindgen]
pub fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[wasm_bindgen]
pub fn set_console_error_panic_hook() {
    console_error_panic_hook::set_once();
}
