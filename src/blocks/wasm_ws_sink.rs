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
use futures::lock::Mutex;
use futures::SinkExt;
use reqwasm::websocket::futures::WebSocket;
use reqwasm::websocket::Message;
use std::marker::PhantomData;
use std::mem::size_of;
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;

// Creating and keeping a WebSocket instance in the WasmWsSink struct didn't work after the first
// send. In combination with the !Send marker in WebSocket (due to Rc<...>), creating a new connection
// every time we want to send new data is less friction for now.

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
        _io: &mut WorkIo,
        sio: &mut StreamIo,
        _mio: &mut MessageIo<Self>,
        _meta: &mut BlockMeta,
    ) -> Result<()> {
        let input = sio.input(0).slice::<u8>();
        let x = sio.input(0).slice::<T>();
        let length = x.len();

        let url_clone = self.url.clone();

        spawn_local(async move {
            let mut x = WebSocket::open(&url_clone).unwrap();
            x.send(Message::Bytes(Vec::from(input))).await.unwrap();
            debug!("WS data sent");
            x.close(None, None).unwrap();
            debug!("WS connection closed");
        });

        sio.input(0).consume(length);
        Ok(())
    }
}
