use capnp::{capability::Promise, traits::ToU16};
use capnp_rpc::{pry, CapabilityServerSet};
use std::path::Path;
use tokio::io::AsyncWriteExt;

use crate::schema::{
    core::{errors_capnp::ErrorCode, files_capnp::CreateMode},
    nodes::{directory_capnp::directory, node_capnp::node},
};

use super::{file::FileImpl, node::NodeImpl};

type DirectoryCapabilitySet = CapabilityServerSet<DirectoryImpl<'static>, directory::Client>;

pub struct DirectoryImpl<'a> {
    path: &'a Path,
    dir_set: &'a DirectoryCapabilitySet,
}

impl<'a> DirectoryImpl<'a> {
    pub fn new(path: &'a Path, dir_set: &DirectoryCapabilitySet) -> DirectoryImpl<'a> {
        DirectoryImpl { path, dir_set }
    }
}

impl<'a> directory::Server for DirectoryImpl<'a> {
    fn lookup(
        &mut self,
        params: directory::LookupParams,
        mut results: directory::LookupResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let name = pry!(pry!(params.get()).get_name());
        let path = self.path.join(name);

        let results = results.get();

        if path.exists() {
            let node = NodeImpl::new(path.as_path());
            let client = capnp_rpc::new_client(node);
            results.set_value(client);
        } else {
            let error = results.init_error();
            error.set_code(ErrorCode::NoEntity.to_u16());
            error.set_message(format!("{name} does not exist").as_str());
            results.set_error(error.into_reader());
        }

        Promise::ok(())
    }

    fn create(
        &mut self,
        params: directory::CreateParams,
        results: directory::CreateResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let params = pry!(params.get());
        let mode = pry!(params.get_mode());
        let name = pry!(params.get_name());
        let path = self.path.join(name);

        let results = results.get();

        if mode == CreateMode::Unchecked {
            // Just return a file handle with no FS checks
            let file = FileImpl::new(path.as_path());
            let client = capnp_rpc::new_client(file);
            results.set_value(client);
            return Promise::ok(());
        }

        if path.exists() {
            let error = results.init_error();
            error.set_code(ErrorCode::Exists.to_u16());
            error.set_message(format!("{name} exists").as_str());
            results.set_error(error.into_reader());
            Promise::ok(())
        } else {
            Promise::from_future(async move {
                // Touch the file
                tokio::fs::File::create(path).await?.shutdown().await?;
                let dir = FileImpl::new(path.as_path());
                let client = capnp_rpc::new_client(dir);
                results.set_value(client);
                Ok(())
            })
        }
    }

    fn mkdir(
        &mut self,
        params: directory::MkdirParams,
        mut results: directory::MkdirResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let params = pry!(params.get());
        let name = pry!(params.get_name());
        let path = self.path.join(name);

        let results = results.get();

        if path.exists() {
            let error = results.init_error();
            error.set_code(ErrorCode::Exists.to_u16());
            error.set_message(format!("{name} exists").as_str());
            results.set_error(error.into_reader());
            Promise::ok(())
        } else {
            Promise::from_future(async move {
                tokio::fs::create_dir(path).await?;
                let dir = DirectoryImpl::new(path.as_path(), self.dir_set);
                let client = self.dir_set.new_client(dir);
                results.set_value(client);
                Ok(())
            })
        }
    }

    fn rmdir(
        &mut self,
        _: directory::RmdirParams,
        _: directory::RmdirResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method directory::Server::rmdir not implemented".to_string(),
        ))
    }

    fn symlink(
        &mut self,
        _: directory::SymlinkParams,
        _: directory::SymlinkResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method directory::Server::symlink not implemented".to_string(),
        ))
    }

    fn readdir(
        &mut self,
        _: directory::ReaddirParams,
        _: directory::ReaddirResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method directory::Server::readdir not implemented".to_string(),
        ))
    }
}

impl<'a> node::Server for DirectoryImpl<'a> {
    fn getattr(
        &mut self,
        _: node::GetattrParams,
        _: node::GetattrResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method node::Server::getattr not implemented".to_string(),
        ))
    }

    fn access(
        &mut self,
        _: node::AccessParams,
        _: node::AccessResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method node::Server::access not implemented".to_string(),
        ))
    }

    fn rename(
        &mut self,
        _: node::RenameParams,
        _: node::RenameResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method node::Server::rename not implemented".to_string(),
        ))
    }

    fn fsstat(
        &mut self,
        _: node::FsstatParams,
        _: node::FsstatResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method node::Server::fsstat not implemented".to_string(),
        ))
    }

    fn commit(
        &mut self,
        _: node::CommitParams,
        _: node::CommitResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method node::Server::commit not implemented".to_string(),
        ))
    }

    fn link(
        &mut self,
        _: node::LinkParams,
        _: node::LinkResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method node::Server::link not implemented".to_string(),
        ))
    }
}
