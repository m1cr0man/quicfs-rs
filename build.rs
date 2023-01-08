use std::collections::HashSet;

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
            use prost::Message;\n\
            use crate::schema::rpc::RpcData;\n\
            use crate::{encode_rpc, decode_rpc, schema_helpers::RpcCodec};\n\
        ",
        );

        let mut method_names = format!(
            "#[derive(Debug)]\n\
            pub enum {}Method {{\n\
                Undefined,\n\
            ",
            service.name
        );

        // TODO use derive macro instead
        let mut method_names_into_str = format!(
            "impl From<{}Method> for String {{\n\
                fn from(method: {}Method) -> Self {{\n\
                    (match method {{\n\
                        {}Method::Undefined => \"Undefined\",\n\
            ",
            service.name, service.name, service.name
        );
        let mut str_into_method_names = format!(
            "impl From<String> for {}Method {{\n\
                fn from(method_str: String) -> Self {{\n\
                    match method_str.as_str() {{\n\
                        &_ => {}Method::Undefined,\n\
            ",
            service.name, service.name
        );

        let mut request_enum_from_rpc = format!(
            "impl RpcCodec<{}Request> for {}Request {{\n\
                fn from_rpc(rpc: RpcData) -> Result<Self, prost::DecodeError> {{\n\
                    let method: {}Method = rpc.method.clone().into();\n\
                    match method {{\n\
            ",
            service.name, service.name, service.name
        );
        let mut response_enum_from_rpc = format!(
            "impl RpcCodec<{}Response> for {}Response {{\n\
                fn from_rpc(rpc: RpcData) -> Result<Self, prost::DecodeError> {{\n\
                    let method: {}Method = rpc.method.clone().into();\n\
                    match method {{\n\
            ",
            service.name, service.name, service.name
        );

        let mut request_enum_to_rpc = format!(
            "fn to_rpc(&self) -> RpcData {{\n\
                match self {{\n\
            ",
        );
        let mut response_enum_to_rpc = format!(
            "fn to_rpc(&self) -> RpcData {{\n\
                match self {{\n\
            ",
        );

        let mut input_types = HashSet::new();
        let mut output_types = HashSet::new();

        for method in service.methods {
            method_names.push_str(&format!("{},\n", method.proto_name));
            method_names_into_str.push_str(&format!(
                "{}Method::{} => \"{}::{}\",\n\
                ",
                service.name, method.proto_name, service.name, method.proto_name
            ));
            str_into_method_names.push_str(&format!(
                "\"{}::{}\" => {}Method::{},\n\
                ",
                service.name, method.proto_name, service.name, method.proto_name
            ));
            input_types.insert(method.input_type.clone());
            output_types.insert(method.output_type.clone());
            request_enum_from_rpc.push_str(&format!(
                "{}Method::{} => decode_rpc!({}, rpc),\n",
                service.name,
                method.proto_name,
                method.input_type // "{}Method::{} => \n\
                                  //     Ok(Self::{}({}::decode(rpc.body)?))\n\
                                  // ,",
                                  // service.name, method.proto_name, method.input_type, method.input_type,
            ));
            response_enum_from_rpc.push_str(&format!(
                "{}Method::{} => decode_rpc!({}, rpc),\n",
                service.name, method.proto_name, method.output_type
            ));
            request_enum_to_rpc.push_str(&format!(
                "Self::{}(v) => encode_rpc!({}Method::{}, v),\n",
                method.input_type, service.name, method.proto_name
            ));
            response_enum_to_rpc.push_str(&format!(
                "Self::{}(v) => encode_rpc!({}Method::{}, v),\n",
                method.output_type, service.name, method.proto_name
            ));
        }

        let default_case = format!("QuicfsMethod::Undefined => Err(prost::DecodeError::new(format!(\"Unrecognised RPC method {{}}\", rpc.method))),");
        request_enum_from_rpc.push_str(&default_case.clone());
        response_enum_from_rpc.push_str(&default_case);

        let mut request_enum = format!("#[derive(Debug)]\npub enum {}Request {{", service.name);
        for typ in input_types.iter() {
            request_enum.push_str(&format!("{}({}),", typ, typ));
        }

        let mut response_enum = format!("#[derive(Debug)]\npub enum {}Response {{", service.name);
        for typ in output_types.iter() {
            response_enum.push_str(&format!("{}({}),", typ, typ));
        }

        method_names.push_str("}");
        buf.push_str(&method_names);
        method_names_into_str.push_str("}).to_string() } }");
        buf.push_str(&method_names_into_str);
        str_into_method_names.push_str("} } }");
        buf.push_str(&str_into_method_names);

        request_enum.push_str("}");
        response_enum.push_str("}");
        buf.push_str(&request_enum);
        buf.push_str(&response_enum);

        request_enum_from_rpc.push_str("} }");
        response_enum_from_rpc.push_str("} }");
        request_enum_from_rpc.push_str(&request_enum_to_rpc);
        response_enum_from_rpc.push_str(&response_enum_to_rpc);
        request_enum_from_rpc.push_str("} } }");
        response_enum_from_rpc.push_str("} } }");
        buf.push_str(&request_enum_from_rpc);
        buf.push_str(&response_enum_from_rpc);

        return;
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
    conf.bytes(&["."]);
    conf.compile_protos(&["schema/quicfs.proto", "schema/rpc.proto"], &["schema/"])
        .unwrap();
}
