use async_std::sync::{Arc, Mutex};
use futures::channel::mpsc;
use futures::stream::StreamExt;
use futures::{Future, SinkExt};
use std::io::Result;

use crate::schema::quicfs::ReaddirResponse;
use crate::schema::quicfs::{QuicfsRequest, QuicfsResponse, ReaddirRequest};

pub struct FileHandle;

type HandlesStore = Arc<Mutex<Vec<FileHandle>>>;

#[derive(Clone)]
struct QuicfsServerHandler {
    handles: HandlesStore,
}

impl QuicfsServerHandler {
    fn new() -> Self {
        Self {
            handles: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn handle_request(&self, request: QuicfsRequest) -> Result<QuicfsResponse> {
        match request {
            QuicfsRequest::FSStat(req) => todo!(),
            QuicfsRequest::Mount(_) => todo!(),
            QuicfsRequest::Getattr(_) => todo!(),
            QuicfsRequest::Readdir(req) => self.handle_readdir(req),
            QuicfsRequest::Read(_) => todo!(),
        }
        .await
    }

    async fn handle_readdir(&self, request: ReaddirRequest) -> Result<QuicfsResponse> {
        // TODO design message passing system for request processing
        Ok(QuicfsResponse::Readdir(ReaddirResponse {
            attributes: Vec::new(),
            eof: true,
            error: "".to_string(),
            offset: 0,
            size: 0,
        }))
    }
}

// The type I represents context passed between the query
// and response for client identification on the caller side.
pub struct QuicfsServer<I> {
    handler: QuicfsServerHandler,
    request_channel: mpsc::Receiver<(I, QuicfsRequest)>,
    response_channel: mpsc::Sender<(I, Result<QuicfsResponse>)>,
}

impl<I> QuicfsServer<I> {
    pub fn new(
        request_channel: mpsc::Receiver<(I, QuicfsRequest)>,
        response_channel: mpsc::Sender<(I, Result<QuicfsResponse>)>,
    ) -> Self {
        let handler = QuicfsServerHandler::new();
        Self {
            handler,
            request_channel,
            response_channel,
        }
    }

    pub fn run(self, concurrency_limit: Option<usize>) -> impl Future<Output = ()> {
        // Extra move here before query so that self becomes owned by the closure
        self.request_channel
            .for_each_concurrent(concurrency_limit, move |(ident, request)| {
                // We need to clone handler before returning our async closure.
                // The async block makes a future which gets returned, and you can't create/return the future
                // multiple times if it takes ownership of something
                let handler = self.handler.clone();
                let mut response_rx = self.response_channel.clone();

                async move {
                    // Don't do anything if the receiver is already gone.
                    // It would be nice if we could return future::Ready outside of this
                    // async closure before even creating it.
                    if response_rx.is_closed() {
                        return;
                    }

                    response_rx
                        .send((ident, handler.handle_request(request).await))
                        .await
                        .expect("Failed to send response");
                }
            })
    }
}
