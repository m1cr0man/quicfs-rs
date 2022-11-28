use std::path::Path;

use capnp::{capability::Promise, traits::ToU16};
use capnp_rpc::pry;
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::schema::{
    core::errors_capnp::ErrorCode,
    nodes::{file_capnp::file, node_capnp::node},
};

pub struct FileImpl<'a> {
    path: &'a Path,
}

impl<'a> FileImpl<'a> {
    pub fn new(path: &'a Path) -> FileImpl<'a> {
        FileImpl { path }
    }
}
use tokio::io::AsyncSeekExt;
impl<'a> file::Server for FileImpl<'a> {
    fn readlink(
        &mut self,
        _: file::ReadlinkParams,
        _: file::ReadlinkResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method file::Server::readlink not implemented".to_string(),
        ))
    }

    // TODO test
    // wat
    // wat
    // wat
    // write the client
    // wat
    // wat
    fn read(
        &mut self,
        params: file::ReadParams,
        mut results: file::ReadResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        let params = pry!(params.get());
        let offset = params.get_offset();
        let count = params.get_count();
        let results = results.get();

        Promise::from_future(async move {
            let options = OpenOptions::new().read(true).write(false);
            match options.open(self.path).await {
                Ok(fh) => {
                    fh.seek(std::io::SeekFrom::Start(offset)).await?;
                    let buf = Vec::with_capacity(count as usize).as_mut();
                    let readlen = fh.read(buf).await?;
                    fh.shutdown().await?;
                    let value = results.init_value();
                    value.set_data(buf);
                    value.set_eof(readlen < count as usize);
                    results.set_value(value.into_reader());
                }
                Err(err) => {
                    let error = results.init_error();
                    error.set_code(ErrorCode::Unknown.to_u16());
                    error.set_message(err.to_string().as_str());
                    results.set_error(error.into_reader());
                }
            };
            Ok(())
        })
    }

    fn write(
        &mut self,
        _: file::WriteParams,
        _: file::WriteResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method file::Server::write not implemented".to_string(),
        ))
    }

    fn remove(
        &mut self,
        _: file::RemoveParams,
        _: file::RemoveResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method file::Server::remove not implemented".to_string(),
        ))
    }

    fn link(
        &mut self,
        _: file::LinkParams,
        _: file::LinkResults,
    ) -> capnp::capability::Promise<(), capnp::Error> {
        capnp::capability::Promise::err(capnp::Error::unimplemented(
            "method file::Server::link not implemented".to_string(),
        ))
    }
}

impl<'a> node::Server for FileImpl<'a> {
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
