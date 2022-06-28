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
use futures::{channel, SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use std::marker::PhantomData;
use std::mem::size_of;
use wasm_bindgen_futures::spawn_local;

//TODO: Create data struct and serialize it with bincode (or similar), then send to server and deserialize
// there.
pub struct WasmWsSink<T> {
    data_sender: channel::mpsc::Sender<(Vec<u8>, channel::oneshot::Sender<bool>)>,
    _p: PhantomData<T>,
}

impl<T: Send + Sync + 'static> WasmWsSink<T> {
    pub fn new(url: String) -> Block {
        let (sender, mut receiver) =
            channel::mpsc::channel::<(Vec<u8>, channel::oneshot::Sender<bool>)>(10);

        spawn_local(async move {
            let mut conn = WebSocket::open(&url).unwrap();
            while let Some((v, sender)) = receiver.next().await {
                if let Err(e) = conn.send(Message::Bytes(v)).await {
                    debug!("{}", e);
                    sender.send(false).unwrap();
                } else {
                    sender.send(true).unwrap();
                    debug!("WS data sent");
                }
            }
        });

        Block::new(
            BlockMetaBuilder::new("WasmWsSink").build(),
            StreamIoBuilder::new()
                .add_input("in", size_of::<T>())
                .build(),
            MessageIoBuilder::<Self>::new().build(),
            WasmWsSink {
                data_sender: sender,
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

        if (items - 2048) >= 2048 {
            io.call_again = true;
        }
        if !v.is_empty() {
            let (sender, receiver) = channel::oneshot::channel::<bool>();
            if let Err(e) = self.data_sender.send((v, sender)).await {
                debug!("{}", e);
            }
            // wait till sending is done
            receiver.await.unwrap();
        }

        Ok(())
    }
}
