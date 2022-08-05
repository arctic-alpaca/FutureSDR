pub mod fft_shift;
pub mod keep_1_in_n;
pub mod wasm;

use gloo_worker::{HandlerId, Worker, WorkerScope};
use shared_utils::Msg;
use wasm_bindgen_futures::spawn_local;

#[cfg(feature = "include_main")]
use gloo_worker::Registrable;
#[cfg(feature = "include_main")]
use wasm_bindgen::prelude::*;

pub struct FftWorker {}

impl Worker for FftWorker {
    type Message = Msg<()>;

    type Input = Vec<i8>;

    type Output = ();

    fn create(_scope: &WorkerScope<Self>) -> Self {
        spawn_local(async move {
            wasm::run().await.unwrap();
        });

        Self {}
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        let Msg::Respond { output, id } = msg;

        scope.respond(id, output);
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, who: HandlerId) {
        spawn_local(async move {
            futuresdr::blocks::wasm_sdr::push_samples(msg).await;
        });
        scope.send_message(Msg::Respond {
            output: (),
            id: who,
        });
    }
}

#[cfg(feature = "include_main")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    FftWorker::registrar()
        //.encoding::<MyCodec>()
        .register();
}
