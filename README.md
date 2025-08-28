# HDF5 Data Loader POC

This is a simple [Foxglove](http://foxglove.dev/) [extension](https://docs.foxglove.dev/docs/visualization/extensions) that provides the building blocks for building an HDF5 data loader.

It registers a VFS that can be used to open files provided by the Foxglove app.
When opening a file it gets some info and creates topics for each of the links in the files.

## Building

Clone this repo with submodules. The hdf5 source is currently included as a submodule.

Install rust with [rustup](https://www.rust-lang.org/tools/install), then install wasm32-wasip1 support:

```
rustup target add wasm32-wasip1
```

Also install the [WASI_SDK](https://github.com/WebAssembly/wasi-sdk).

Export the path to your WASI SDK:

```sh
export WASI_SDK_PATH="/opt/wasi-sdk/"
```

Then to build the rust code and generate the extension file:

```
npm install
npm run package
```

These steps will produce a `.foxe` file you can install as an extension from the Foxglove settings page.
