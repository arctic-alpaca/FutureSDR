pub use fft_shift::FftShift;
mod fft_shift;
mod keep_1_in_n;
pub mod wasm;
pub mod worker;

use futuresdr::blocks::Apply;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::Block;
use wasm_bindgen::prelude::*;

pub fn lin2db_block() -> Block {
    Apply::new(|x: &f32| 10.0 * x.log10())
}

pub fn power_block() -> Block {
    Apply::new(|x: &Complex32| x.norm())
}

#[wasm_bindgen]
pub fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
