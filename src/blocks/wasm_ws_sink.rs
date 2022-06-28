use crate::anyhow::Result;
use crate::runtime::Block;
use crate::runtime::BlockMeta;
use crate::runtime::BlockMetaBuilder;
use crate::runtime::Kernel;
use crate::runtime::MessageIo;
use crate::runtime::MessageIoBuilder;
use crate::runtime::StreamIo;
use crate::runtime::StreamIoBuilder;
use crate::runtime::WorkIo;
use futures::SinkExt;
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use std::marker::PhantomData;
use std::mem::size_of;
use wasm_bindgen_futures::spawn_local;

// Creating and keeping a WebSocket instance in the WasmWsSink struct didn't work after the first
// send. In combination with the !Send marker in WebSocket (due to Rc<...>), creating a new connection
// every time we want to send new data is less friction for now.

//TODO: spawn local to handle connection completely, communicate via futures::sync::mpsc::{...}

//TODO: Create data struct and serialize it with bincode (or similar), then send to server and deserialize
// there.
pub struct WasmWsSink<T> {
    url: String,
    _p: PhantomData<T>,
}

impl<T: Send + Sync + 'static> WasmWsSink<T> {
    pub fn new(url: String) -> Block {
        Block::new(
            BlockMetaBuilder::new("WasmWsSink").build(),
            StreamIoBuilder::new()
                .add_input("in", size_of::<T>())
                .build(),
            MessageIoBuilder::<Self>::new().build(),
            WasmWsSink {
                url,
                _p: PhantomData,
            },
        )
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> Kernel for WasmWsSink<T> {
    async fn work(
        &mut self,
        io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        debug!("start wasm_ws_sink");
        //let input = sio.input(0).slice::<u8>();
        //let input2 = sio.input(0).slice::<T>();
        //let length = input2.len();

        let i = sio.input(0).slice::<u8>();
        debug_assert_eq!(i.len() % size_of::<T>(), 0);
        if i.is_empty() {
            return Ok(());
        }

        if sio.input(0).finished() {
            io.finished = true;
        }

        let mut v = Vec::new();
        let item_size = size_of::<T>();
        let items = i.len() / item_size;
        if 2048 <= items {
            v.extend_from_slice(&i[0..(2048 * item_size)]);
            sio.input(0).consume(2048);
        }
        if !v.is_empty() {
            let url_clone = self.url.clone();

            // We need spawn_local because WebSocket is !Send and Kernel requires Send.
            // Forking gloo_net and replacing Rc with Arc and RefCell with Mutex is not a viable option
            // as more modifications are needed.
            spawn_local(async move {
                let mut x = WebSocket::open(&url_clone).unwrap();
                x.send(Message::Bytes(v)).await.unwrap();
                debug!("WS data sent");

                x.close(None, None).unwrap();
                debug!("WS connection closed");
            });
        }
        //sio.input(0).consume(length);
        debug!("end wasm_ws_sink");
        Ok(())
    }
}
