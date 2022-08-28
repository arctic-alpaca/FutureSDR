use futuresdr::anyhow::Result;
use futuresdr::blocks::Apply;
use futuresdr::blocks::Fft;
use futuresdr::blocks::WasmSdr;
use futuresdr::blocks::WasmWsSink;
use futuresdr::num_complex::Complex32;
use futuresdr::runtime::buffer::slab::Slab;
use futuresdr::runtime::Block;
use futuresdr::runtime::Flowgraph;
use futuresdr::runtime::Runtime;

use crate::fft_shift::FftShift;

pub fn lin2db_block() -> Block {
    Apply::new(|x: &f32| 10.0 * x.log10())
}
pub fn power_block() -> Block {
    Apply::new(|x: &Complex32| x.norm())
}

pub async fn run(ws_url: String) -> Result<()> {
    let mut fg = Flowgraph::new();

    let src = fg.add_block(WasmSdr::new());
    let fft = fg.add_block(Fft::new());
    let power = fg.add_block(power_block());
    let log = fg.add_block(lin2db_block());
    let shift = fg.add_block(FftShift::<f32>::new());
    //let keep = fg.add_block(Keep1InN::new(0.1, 40));
    let snk = fg.add_block(WasmWsSink::<f32>::new(
        
        ws_url,
        shared_utils::FFT_CHUNKS_PER_WS_TRANSFER,
    ));

    fg.connect_stream_with_type(src, "out", fft, "in", Slab::with_config(65536, 2, 0))?;
    fg.connect_stream_with_type(fft, "out", power, "in", Slab::with_config(65536, 2, 0))?;
    fg.connect_stream_with_type(power, "out", log, "in", Slab::with_config(65536, 2, 0))?;
    fg.connect_stream_with_type(log, "out", shift, "in", Slab::with_config(65536, 2, 0))?;
    //fg.connect_stream_with_type(shift, "out", keep, "in", Slab::with_config(65536, 2, 0))?;
    //fg.connect_stream_with_type(keep, "out", snk, "in", Slab::with_config(65536, 2, 0))?;
    fg.connect_stream_with_type(shift, "out", snk, "in", Slab::with_config(65536, 2, 0))?;

    Runtime::new().run_async(fg).await?;

    Ok(())
}
