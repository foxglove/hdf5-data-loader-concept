use std::{env, path::PathBuf};

fn main() {
    let wasi_sdk_dir = env::var("WASI_SDK_PATH").unwrap_or_else(|_| "/opt/wasi-sdk/".into());
    let wasi_sdk_dir = PathBuf::from(wasi_sdk_dir);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest dir");
    let package_root = PathBuf::from(manifest_dir);

    let dst = cmake::Config::new("hdf5")
        .define(
            "CMAKE_TOOLCHAIN_FILE",
            wasi_sdk_dir
                .join("share/cmake/wasi-sdk-p1.cmake")
                .to_string_lossy()
                .to_string(),
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
        .cflag(format!(
            "-include {}",
            package_root.join("lck.h").to_string_lossy()
        ))
        .cflag(format!(
            "-include {}",
            package_root.join("tzset.h").to_string_lossy()
        ))
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
        .clang_arg(format!(
            "--sysroot={}",
            wasi_sdk_dir.join("share/wasi-sysroot").to_string_lossy()
        ))
        // needed to get the vfs symbols
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
    println!("cargo:rustc-link-lib=static=hdf5");
}
