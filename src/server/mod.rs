use capnp::capability::Promise;

use crate::schema::server_capnp::server;

pub struct QuicfsServer {}

impl server::Server for QuicfsServer {
    // fn mount(&mut self, params: server::MountParams, mut results: server::MountResults) -> capnp::capability::Promise<(), capnp::Error> {
    //     results.get().set_value(value)
    // }
    fn null(&mut self, _: server::NullParams, _: server::NullResults) -> Promise<(), capnp::Error> {
        Promise::ok(())
    }
}
