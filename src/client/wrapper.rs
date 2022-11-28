use crate::schema::nodes::directory_capnp;

pub struct QuicFSAPIWrapper {
    client: directory_capnp::directory::Client,
}
