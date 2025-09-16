#!/usr/bin/env bash

cmake -S . -B build -DCMAKE_INSTALL_PREFIX=dist
cmake --build build
cmake --install build
