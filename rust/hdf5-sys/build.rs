use std::{env, path::PathBuf};

fn main() {
    let wasi_sdk_dir = env::var("WASI_SDK_PATH").unwrap_or_else(|_| "/opt/wasi-sdk/".into());
    let wasi_sdk_dir = PathBuf::from(wasi_sdk_dir);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest dir");
    let package_root = PathBuf::from(manifest_dir);

    // let mut config = cmake::Config::new("/Users/bennett/git/hdf5/");
    let mut config = cmake::Config::new("/Users/bennett/git/hdf5_with_plugins/");

    config
        // .define("BUILD_SHARED_LIBS", "on")
        // .define("BUILD_STATIC_LIBS", "on")
        // .define("HDF_PACKAGE_NAMESPACE", "hdf5::")
        // .define("HDF5_BUILD_EXAMPLES", "off")
        // .define("HDF5_BUILD_TOOLS", "off")
        // .define("HDF5_BUILD_UTILS", "off")
        // .define("HDF5_BUILD_CPP_LIB", "off")
        // .define("HDF5_ALLOW_EXTERNAL_SUPPORT", "TGZ")
        // .define("HDF5_ENABLE_ZLIB_SUPPORT", "on")
        // .define("ZLIB_USE_EXTERNAL", "on")
        // .define("HDF5_USE_ZLIB_STATIC", "on")
        // .define("HDF5_ENABLE_PLUGIN_SUPPORT", "on")
        // .define("PLUGIN_USE_EXTERNAL", "on")
        // .define("HDF5_NO_PACKAGES", "off")
        // .define("H5PL_ALLOW_EXTERNAL_SUPPORT", "TGZ")
        // .define("H5PL_BUILD_TESTING", "off")
        // .define("ENABLE_BLOSC", "off")
        // .define("ENABLE_BLOSC2", "off")
        // .define("ENABLE_BZIP2", "off")
        // .define("ENABLE_LZ4", "off")
        // .define("ENABLE_ZFP", "off")
        // .define("ENABLE_ZSTD", "off")
        // .define("ENABLE_BSHUF", "off")

        // .define("H5PL_CPACK_ENABLE", "on")
        // // .define("ZLIB_GIT_URL", "https://github.com/madler/zlib.git")
        // // .define("ZLIB_GIT_TAG", "v1.3.1")
        // // .define("HDF5_ENABLE_PLUGIN_SUPPORT", "on")
        // // .define("PLUGIN_USE_LOCALCONTENT", "off")
        // // .define("HDF5_ENABLE_PLUGIN_SUPPORT", "on")
        // // .define("HDF5_ENABLE_SZIP_SUPPORT", "on")
        // // .define("HDF5_ENABLE_SZIP_SUPPORT", "on")
        // .define("HDF5_PACKAGE_EXTLIBS", "on")
        // // .define("HDF5_ALLOW_EXTERNAL_SUPPORT ", "GIT")
        // // .define("PLUGIN_USE_LOCALCONTENT", "0")
        // // // .define("PLUGIN_USE_EXTERNAL", "1")
        // // .define("ZLIB_USE_LOCAL_CONTENT", "0")
        // // .define("SZIP_USE_EXTERNAL", "1")
        // // .define("ZLIB_USE_EXTERNAL", "1")
        // // .define("HDF5_EXTERNALLY_CONFIGURED", "1")
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
    println!("cargo:rustc-link-search=native={}/lib/plugin", dst.display());
    println!("cargo:rustc-link-lib=static=hdf5");
    println!("cargo:rustc-link-lib=static=zlib-static");
    println!("cargo:rustc-link-lib=static=h5lzf");
    println!("cargo:rustc-link-lib=static=lzf");
}
