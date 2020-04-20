// use std::env;
// use std::fs;
// use std::path::PathBuf;
// use syn::{parse_macro_input, LitStr};

mod generation;
pub use generation::*;

// #[proc_macro]
// pub fn feather_protocol(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let version = parse_macro_input!(input as LitStr).value();
//     let path = format!(
//         "{out}/minecraft-data/data/pc/{version}/",
//         out = env!("OUT_DIR"),
//         version = version
//     );

//     let path = PathBuf::from(path);

//     let protocol = parsing::parse(path).expect("Failed to parse.");
//     let generated = generation::generate(protocol);

//     // integration::integrate(&[protocol])
//     // Parse
//     // Integrate
//     // Generate
//     generated.into()
// }
