extern crate prost_build;
extern crate prost_types;

struct QuicfsServiceGenerator {}

fn is_repeated_response(options: Vec<prost_types::UninterpretedOption>) -> bool {
    println!("cargo:warning=hi {:?}", options);
    for opt in options {
        if opt.name.first().unwrap().name_part == "repeated_response" {
            println!("cargo:warning={:?}", opt.aggregate_value);
            return opt.identifier_value() == "true";
        }
    }
    return false;
}

impl prost_build::ServiceGenerator for QuicfsServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        buf.insert_str(
            0,
            "\
            use crate::protocol::{ProtobufProtocol, Protocol};\n\
            use crate::server::{Handler, HandlerError};\n\
            use prost::Message;\n\
            use std::pin::Pin;\n\
        ",
        );
        let mut client = format!(
            "pub struct {}Client<T: crate::transport::Transport> {{",
            service.name
        );
        client.push_str(&format!("transport: T"));
        client.push_str("}");
        client.push_str(&format!(
            "impl<T: crate::transport::Transport> {}Impl for {}Client<T> {{",
            service.name, service.name
        ));

        let mut handler = format!(
            "\
            impl<T: {}Impl + Clone + Unpin> Handler for T {{\
                async fn handle(mut self: Pin<&mut Self>, proto: Pin<&mut ProtobufProtocol<crate::transport::QuicTransportPeer>>) -> Result<(), HandlerError> {{\
                    let proto = proto.get_mut();
                    if let Ok(msg) = proto.read_message::<crate::schema::rpc::RpcData>().await {{\
                        match msg.method.as_str() {{\
            ",
            service.name
        );

        // Generate server impl
        buf.push_str(&format!("pub trait {}Impl {{", service.name));

        for method in service.methods {
            // FIXME when uninterpreted_option is populated.
            // See https://github.com/tokio-rs/prost/pull/591
            let result = match is_repeated_response(method.options.uninterpreted_option) {
                true => format!("futures::stream::Stream<{}>", method.output_type),
                false => format!("{}", method.output_type),
            };
            let method_sig = format!(
                "async fn {}(&mut self, request: {}) -> {}",
                method.name, method.input_type, result
            );

            // Trait method
            buf.push_str(&format!("{};", method_sig));

            // Client method
            client.push_str(&format!("{} {{", method_sig));
            // client.push_str(&format!("{}{{}}", method.output_type));
            client.push_str("todo!()");
            client.push_str("}");

            // Handler method
            handler.push_str(&format!(
                "\"{}::{}\" => {{\
                    let req = {}::decode(msg.body.as_ref())\
                        .map_err(HandlerError::from)?;\
                    let resp = self.{}(req).await;\
                    proto.write_message(resp).await\
                        .map_err(HandlerError::from)?;\
                }}",
                service.name, method.name, method.input_type, method.name,
            ));
        }

        buf.push_str("}");
        client.push_str("}");
        handler.push_str("};\n}Ok(())\n}\n}");

        // Append the client and handler
        buf.push_str(&client);
        buf.push_str(&handler);
    }
}

fn main() {
    let mut conf = prost_build::Config::new();
    conf.out_dir("src/schema");
    conf.include_file("mod.rs");
    conf.service_generator(Box::new(QuicfsServiceGenerator {}));
    conf.compile_protos(&["schema/quicfs.proto", "schema/rpc.proto"], &["schema/"])
        .unwrap();
}
