use std::{env, path::PathBuf};

fn main() {
    let wasi_sdk_dir = env::var("WASI_SDK_PATH").unwrap_or_else(|_| "/opt/wasi-sdk/".into());
    let wasi_sdk_dir = PathBuf::from(wasi_sdk_dir);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest dir");
    let package_root = PathBuf::from(manifest_dir);

    let mut config = cmake::Config::new("hdf5_with_plugins");

    config
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("H5_HAVE_GETPWUID", "off")
        .define("H5_HAVE_SIGNAL", "off")
        .define("H5_HAVE_FEATURES_H", "off");

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target_arch == "wasm32" {
        config
            .define(
                "CMAKE_TOOLCHAIN_FILE",
                wasi_sdk_dir
                    .join("share/cmake/wasi-sdk-p1.cmake")
                    .to_string_lossy()
                    .to_string(),
            )
            .cflag("-mllvm -wasm-enable-sjlj")
            .cflag("-D_WASI_EMULATED_SIGNAL")
            // .cflag("-lwasi-emulated-signal")
            .cflag(format!(
                "-include {}",
                package_root.join("lck.h").to_string_lossy()
            ))
            .cflag(format!(
                "-include {}",
                package_root.join("tzset.h").to_string_lossy()
            ));
    }

    let dst = config.build();

    println!("cargo:warning=output is {}", dst.display());

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include", dst.display()));

    if target_arch == "wasm32" {
        builder = builder.clang_arg(format!(
            "--sysroot={}",
            wasi_sdk_dir.join("share/wasi-sysroot").to_string_lossy()
        ));
    }

    let bindings = builder
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
    println!(
        "cargo:rustc-link-search=native={}/lib/plugin",
        dst.display()
    );
    println!("cargo:rustc-link-lib=static=hdf5");
    println!("cargo:rustc-link-lib=static=zlib-static");
    println!("cargo:rustc-link-lib=static=aec");
    println!("cargo:rustc-link-lib=static=szaec");
    println!("cargo:rustc-link-lib=static=h5lzf");
    println!("cargo:rustc-link-lib=static=lzf");
}
