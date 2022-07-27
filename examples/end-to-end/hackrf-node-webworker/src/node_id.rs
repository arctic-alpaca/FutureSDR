use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
use gloo_worker::{Codec, Registrable};
use gloo_worker::{HandlerId, Spawnable, Worker, WorkerBridge, WorkerScope};
pub use keep_1_in_n::Keep1InN;
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console::log_1;
use web_sys::{WorkerOptions, WorkerType};

#[wasm_bindgen]
pub fn set_node_id_input_handler() {
    let window = web_sys::window().expect("should have a window in this context");
    let document = window.document().expect("window should have a document");
    let a = Closure::<dyn FnMut()>::new(move || {
        clicks += 1;
        num_clicks.set_inner_html(&clicks.to_string());
    });
    document
        .get_element_by_id("green-square")
        .expect("should have #green-square on the page")
        .dyn_ref::<HtmlElement>()
        .expect("#green-square be an `HtmlElement`")
        .set_onclick(Some(a.as_ref().unchecked_ref()));

    // See comments in `setup_clock` above for why we use `a.forget()`.
    a.forget();
}
