use std::env;
use std::fs;
use std::path::PathBuf;
use syn::{parse_macro_input, LitStr};
use feather_protocol_codegen::*;

#[proc_macro]
pub fn protocol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let protocol = parse_macro_input!(input as LitStr).value();
    let path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is unset"));
    let path = path.join(protocol);

    let file = fs::File::open(path).unwrap();

    let protocol_de: Protocol = ron::de::from_reader(&file).unwrap();

    let protocol = ProtocolGenerator::generate(protocol_de);

    protocol.into()
}
