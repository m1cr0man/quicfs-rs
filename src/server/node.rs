use std::path::Path;

use capnp::capability::Promise;

use crate::schema::nodes::node_capnp::node;

pub struct NodeImpl<'a> {
    path: &'a Path,
}

impl<'a> NodeImpl<'a> {
    pub fn new(path: &'a Path) -> NodeImpl<'a> {
        NodeImpl { path }
    }
}

impl<'a> node::Server for NodeImpl<'a> {
    fn link(
        &mut self,
        params: node::LinkParams,
        _results: node::LinkResults,
    ) -> Promise<(), capnp::Error> {
        Promise::from_future(async move {
            let params = params.get()?;
            let name = params.get_name()?;
            let link_name = self.path.parent().unwrap().join(name);

            let target_node = params.get_node()?;

            // let target_path = target_node
            //     .path_request()
            //     .send()
            //     .promise
            //     .await?
            //     .get()?
            //     .get_path()?;

            tokio::fs::symlink("", link_name).await?;
            Ok(())
        })
    }
}
