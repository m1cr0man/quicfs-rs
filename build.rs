extern crate prost_build;
extern crate prost_types;

struct QuicfsServiceGenerator {}

impl prost_build::ServiceGenerator for QuicfsServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        buf.insert_str(
            0,
            "\
            use prost::Message;\n\
            use crate::schema::rpc::RpcData;\n\
            use crate::{encode_rpc, schema_helpers::{RpcCodec, decode_rpc}};\n\
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
            ",
            service.name
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

        let mut request_enum = format!("#[derive(Debug)]\npub enum {}Request {{", service.name);
        let mut response_enum = format!("#[derive(Debug)]\npub enum {}Response {{", service.name);

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
            request_enum.push_str(&format!("{}({}),", method.proto_name, method.input_type));
            response_enum.push_str(&format!("{}({}),", method.proto_name, method.output_type));
            request_enum_from_rpc.push_str(&format!(
                "{}Method::{} => decode_rpc(rpc).map(|v| Self::{}(v)),\n",
                service.name,
                method.proto_name,
                method.proto_name, // "{}Method::{} => \n\
                                   //     Ok(Self::{}({}::decode(rpc.body)?))\n\
                                   // ,",
                                   // service.name, method.proto_name, method.input_type, method.input_type,
            ));
            response_enum_from_rpc.push_str(&format!(
                "{}Method::{} => decode_rpc(rpc).map(|v| Self::{}(v)),\n",
                service.name, method.proto_name, method.proto_name
            ));
            request_enum_to_rpc.push_str(&format!(
                "Self::{}(v) => encode_rpc!({}Method::{}, v),\n",
                method.proto_name, service.name, method.proto_name
            ));
            response_enum_to_rpc.push_str(&format!(
                "Self::{}(v) => encode_rpc!({}Method::{}, v),\n",
                method.proto_name, service.name, method.proto_name
            ));
        }

        let default_case = format!("QuicfsMethod::Undefined => Err(prost::DecodeError::new(format!(\"Unrecognised RPC method {{}}\", rpc.method))),");
        request_enum_from_rpc.push_str(&default_case.clone());
        response_enum_from_rpc.push_str(&default_case);

        method_names.push_str("}");
        buf.push_str(&method_names);
        method_names_into_str.push_str("}).to_string() } }");
        buf.push_str(&method_names_into_str);
        str_into_method_names.push_str(&format!(
            "&_ => {}Method::Undefined, }} }} }}",
            service.name
        ));
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
