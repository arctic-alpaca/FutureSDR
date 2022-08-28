pub mod fft_shift;
pub mod keep_1_in_n;
pub mod wasm;

use gloo_worker::{HandlerId, Worker, WorkerScope};
use shared_utils::ToWorker;
use wasm_bindgen_futures::spawn_local;

#[cfg(feature = "include_main")]
use gloo_worker::Registrable;

#[cfg(feature = "include_main")]
use wasm_bindgen::prelude::*;

pub struct FftWorker {}

impl Worker for FftWorker {
    type Message = ();

    type Input = shared_utils::ToWorker;

    type Output = ();

    fn create(_scope: &WorkerScope<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _scope: &WorkerScope<Self>, _msg: Self::Message) {}

    fn received(&mut self, _scope: &WorkerScope<Self>, msg: Self::Input, _who: HandlerId) {
        match msg {
            ToWorker::Data { data } => {
                spawn_local(async move {
                    futuresdr::blocks::wasm_sdr::push_samples(data).await;
                });
            }
            ToWorker::ApplyConfig { config } => {
                let ws_url = format!(
                    "ws://127.0.0.1:3000/node/api/data/fft/{}/{}/{}/{}/{}",
                    config.freq, config.amp, config.lna, config.vga, config.sample_rate
                );
                spawn_local(async move {
                    wasm::run(ws_url).await.unwrap();
                });
            }
        }
    }
}

#[cfg(feature = "include_main")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    FftWorker::registrar().register();
}
