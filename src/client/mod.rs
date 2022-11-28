use std::path::PathBuf;

pub mod wrapper;

use crate::schema::core::models_capnp;
use crate::schema::nodes::directory_capnp;

pub struct CommandLineProcessor {
    dir: PathBuf,
    client: directory_capnp::directory::Client,
}

impl CommandLineProcessor {
    pub fn new(client: directory_capnp::directory::Client) -> CommandLineProcessor {
        CommandLineProcessor {
            dir: PathBuf::new(),
            client,
        }
    }

    pub async fn list(&self) -> Result<(), ::capnp::Error> {
        let mut request = self.client.readdir_request();
        let reply = request.send().promise.await.unwrap();
        let reply = reply.get().unwrap();
        match reply.which().unwrap() {
            models_capnp::result::Which::Value(val) => match val {
                Ok(reader) => {
                    let nodes = reader.into_iter();
                    for node in nodes {
                        let mut request = node.unwrap().getattr_request();
                        let reply = request.send().promise.await.unwrap();
                        match reply.get()?.which()? {
                            models_capnp::result::Which::Value(attrs) => attrs?.get_atime(),
                            models_capnp::result::Which::Error(_) => {}
                        }
                    }
                }
                Err(_) => todo!(),
            },
            models_capnp::result::Which::Error(err) => todo!(),
        };
        Ok(())
    }

    pub async fn process_command(&self, input: String) -> Result<(), ::capnp::Error> {
        let mut parts = input.split(" ");
        match parts.next() {
            Some("ls") => self.list().await?,
            None => {}
        };
        Ok(())
    }
}
