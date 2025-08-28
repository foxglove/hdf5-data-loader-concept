use std::{env, path::PathBuf};

fn main() {
    let dst = cmake::Config::new("hdf5")
        .define(
            "CMAKE_TOOLCHAIN_FILE",
            "/opt/wasi-sdk/share/cmake/wasi-sdk-p1.cmake",
        )
        .define("BUILD_SHARED_LIBS", "off")
        .define("HDF5_BUILD_EXAMPLES", "off")
        .define("HDF5_BUILD_TOOLS", "off")
        .define("HDF5_BUILD_UTILS", "off")
        .define("HDF5_BUILD_CPP_LIB", "off")
        .define("HDF5_ENABLE_Z_LIB_SUPPORT", "off")
        .define("HDF5_ENABLE_SZIP_SUPPORT", "off")
        .define("SZIP_USE_EXTERNAL", "0")
        .define("ZLIB_USE_EXTERNAL", "0")
        .define("HDF5_EXTERNALLY_CONFIGURED", "1")
        .define("H5_HAVE_GETPWUID", "off")
        .define("H5_HAVE_SIGNAL", "off")
        .define("H5_HAVE_FEATURES_H", "off")
        .cflag("-mllvm -wasm-enable-sjlj")
        .cflag("-D_WASI_EMULATED_SIGNAL")
        .cflag("-lwasi-emulated-signal")
        .cflag("-include /home/bennett/foxglove/hdf5-wasm-poc/hdf5-sys/lck.h")
        .cflag("-include /home/bennett/foxglove/hdf5-wasm-poc/hdf5-sys/tzset.h")
        .build();

    println!("cargo:warning=output is {}", dst.display());

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include", dst.display()))
        .clang_arg("--sysroot=/opt/wasi-sdk/share/wasi-sysroot")
        .clang_arg("-DH5_BUILT_AS_DYNAMIC_LIB")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=hdf5_debug");
}
