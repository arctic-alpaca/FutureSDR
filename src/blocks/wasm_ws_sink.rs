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
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem::size_of;
use wasm_bindgen_futures::spawn_local;

pub struct WasmWsSink<T> {
    data_sender: channel::mpsc::Sender<Vec<u8>>,
    data_storage: Vec<u8>,
    iterations_per_send: usize,
    _p: PhantomData<T>,
}

impl<T: Send + Sync + 'static> WasmWsSink<T> {
    pub fn new(url: String, iterations_per_send: usize) -> Block {
        let (sender, mut receiver) = channel::mpsc::channel::<Vec<u8>>(1);

        spawn_local(async move {
            let mut conn = WebSocket::open(&url).unwrap();
            while let Some(v) = receiver.next().await {
                if let Err(e) = conn.send(Message::Bytes(v)).await {
                    //debug!("{}", e);
                    //sender.send(false).unwrap();
                } else {
                    //sender.send(true).unwrap();
                    //debug!("WS data sent");
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
                data_storage: Vec::new(),
                iterations_per_send,
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

        let items_to_process_per_run = 2048;

        if i.is_empty() {
            return Ok(());
        }

        if sio.input(0).finished() {
            io.finished = true;
        }

        let mut v = Vec::new();
        let item_size = size_of::<T>();
        let items = i.len() / item_size;
        if items_to_process_per_run <= items {
            v.extend_from_slice(&i[0..(items_to_process_per_run * (item_size / size_of::<u8>()))]);
            sio.input(0).consume(items_to_process_per_run);
        }

        if (items - items_to_process_per_run) >= items_to_process_per_run {
            io.call_again = true;
        }
        if !v.is_empty() {
            self.data_storage.append(&mut v);
            if self.data_storage.len()
                >= items_to_process_per_run
                    * self.iterations_per_send
                    * (item_size / size_of::<u8>())
            {
                let mut movable_vector = Vec::with_capacity(
                    items_to_process_per_run
                        * self.iterations_per_send
                        * (item_size / size_of::<u8>()),
                );
                std::mem::swap(&mut self.data_storage, &mut movable_vector);
                if let Err(e) = self.data_sender.send(movable_vector).await {
                    //debug!("{}", e);
                }
            }
        }

        Ok(())
    }
}
