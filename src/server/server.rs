use crate::schema::server_capnp::server;
use capnp::capability::Promise;
use capnp_rpc::{pry, CapabilityServerSet};
use std::path::Path;

use crate::schema::nodes::directory_capnp::directory;

use super::directory::DirectoryImpl;

pub struct QuicfsServerImpl {
    // TODO do I _have_ to use static?
    dir_set: CapabilityServerSet<DirectoryImpl<'static>, directory::Client>,
}

impl QuicfsServerImpl {
    pub fn new() -> QuicfsServerImpl {
        QuicfsServerImpl {
            dir_set: CapabilityServerSet::new(),
        }
    }
}

impl server::Server for QuicfsServerImpl {
    fn mount(
        &mut self,
        params: server::MountParams,
        mut results: server::MountResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let path = pry!(pry!(params.get()).get_path());
        let dir = DirectoryImpl::new(Path::new(path), &self.dir_set);
        let dir_client = self.dir_set.new_client(dir);
        // return a handle to the root directory
        results.get().set_value(dir_client);
        Promise::ok(())
    }

    fn null(
        &mut self,
        _params: server::NullParams,
        _results: server::NullResults,
    ) -> Promise<(), capnp::Error> {
        Promise::ok(())
    }
}
