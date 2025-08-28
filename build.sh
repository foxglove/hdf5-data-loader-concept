#!/usr/bin/env bash

bindgen ./hdf5.c -o ./src/hdf5_bindings.rs

cargo build
