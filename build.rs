fn main() {}

// use std::env;
// use std::path::{PathBuf, absolute};
// use bindgen;
// 
// fn main() {
//     println!(
//         "cargo:rustc-link-search=native={}",
//         absolute("./wasm-lib/lib/")
//             .unwrap()
//             .to_string_lossy()
//     );
//     println!("cargo:rustc-link-lib=static=hdf5");
//     println!("cargo:rerun-if-changed=wrapper.h");
// 
//     // The bindgen::Builder is the main entry point
//     // to bindgen, and lets you build up options for
//     // the resulting bindings.
//     let mut bindings = bindgen::Builder::default()
//         // The input header we would like to generate
//         // bindings for.
//         .header("wrapper.h")
//         .clang_arg("-DH5_BUILT_AS_DYNAMIC_LIB")
//         // Tell cargo to invalidate the built crate whenever any of the
//         // included header files changed.
//         .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
//         .clang_arg("-I./wasm-lib/include/");
// 
//     let is_wasm = env::var("TARGET").map_or(false, |target| target.starts_with("wasm32-"));
// 
//     if is_wasm {
//         bindings = bindings.clang_arg("--sysroot=/opt/wasi-sdk/share/wasi-sysroot");
//     }
// 
//     // Write the bindings to the $OUT_DIR/bindings.rs file.
//     let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
// 
//     bindings
//         .generate()
//         // Unwrap the Result and panic on failure.
//         .expect("Unable to generate bindings")
//         .write_to_file(out_path.join("bindings.rs"))
//         .expect("Couldn't write bindings!");
// }
