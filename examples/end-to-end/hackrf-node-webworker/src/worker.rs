use crate::wasm;
use gloo_worker::WorkerBridge;
use gloo_worker::{Codec, HandlerId, Registrable, Spawnable, Worker, WorkerScope};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::console::log_1;
use web_sys::{WorkerOptions, WorkerType};

thread_local! {
    static BRIDGES: RefCell<HashMap<String, WorkerBridge<FutureSdrWorker>>> = RefCell::new(HashMap::new());
}

#[derive(Debug)]
pub enum Msg<T> {
    Respond { output: T, id: HandlerId },
}

pub struct FutureSdrWorker {}

impl Worker for FutureSdrWorker {
    type Message = Msg<()>;

    type Input = Vec<i8>;

    type Output = ();

    fn create(_scope: &WorkerScope<Self>) -> Self {
        spawn_local(async move {
            wasm::run_fg().await;
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

/// This skips bincode serialization and goes straight to a JsValue. Using serde_json would be the other option.
/// Performance isn't clear, in limited testing this performed better than using bincode. No testing
/// between serde_wasm_bindgen and serde_json has been done.
/// Proper measurements can be performed by implementing all potential codecs with time used printouts
/// https://rustwasm.github.io/docs/wasm-bindgen/examples/performance.html?highlight=profile#web-sys--performancenow
pub struct MyCodec {}

impl Codec for MyCodec {
    fn encode<I>(input: I) -> JsValue
    where
        I: serde::ser::Serialize,
    {
        serde_wasm_bindgen::to_value(&input).expect("can't serialize an worker message")
    }

    fn decode<O>(input: JsValue) -> O
    where
        O: for<'de> serde::de::Deserialize<'de>,
    {
        serde_wasm_bindgen::from_value(input).expect("can't deserialize an worker message")
    }
}

#[wasm_bindgen]
pub async fn input_samples(samples: Vec<i8>) {
    BRIDGES.with(|x| {
        for bridge in x.borrow_mut().values() {
            bridge.send(samples.clone());
        }
    });
}

#[wasm_bindgen]
pub async fn create_bridges() {
    let mut options = WorkerOptions::new();
    options.type_(WorkerType::Module);
    let mut spawner = FutureSdrWorker::spawner();
    spawner.encoding::<MyCodec>();
    let bridge = spawner.spawn_with_options_and_js_file("/worker.js", &options);
    BRIDGES.with(|x| {
        x.borrow_mut().insert("stuff".to_owned(), bridge);
    });
    let bridge = spawner.spawn_with_options_and_js_file("/worker.js", &options);
    BRIDGES.with(|x| {
        x.borrow_mut().insert("stuff1".to_owned(), bridge);
    });
}

#[wasm_bindgen]
pub fn run_worker() {
    console_error_panic_hook::set_once();

    log_1(&"run_in_worker".into());
    FutureSdrWorker::registrar().register();
}
