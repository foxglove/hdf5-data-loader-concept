cmake_policy(SET CMP0079 NEW)
cmake_minimum_required(VERSION 3.24)

project (H5COMBINED C)

include(FetchContent)
include(ExternalProject)

set (HDF5_BUILD_TOOLS NO CACHE BOOL "" FORCE)
set (HDF_PACKAGE_NAMESPACE "hdf5::" CACHE STRING "" FORCE)

FetchContent_Declare(
  hdf5
  URL https://github.com/HDFGroup/hdf5/releases/download/hdf5_1.14.6/hdf5-1.14.6.tar.gz
)

set (TEST_LFS_WORKS_RUN ON CACHE BOOL "" FORCE)

set (USE_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set (BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set (H5DF_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set (H5PL_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)

set (HDF5_ALLOW_EXTERNAL_SUPPORT "TGZ" CACHE STRING "" FORCE)

set (HDF5_ENABLE_Z_LIB_SUPPORT ON CACHE BOOL "" FORCE)
set (HDF5_PACKAGE_EXTLIBS ON CACHE BOOL "" FORCE)

set (HDF5_EXPORTED_TARGETS "hdf5-static" CACHE BOOL "" FORCE)
set (ZLIB_EXPORTED_TARGETS "zlib-static" CACHE BOOL "" FORCE)
set (LIBAEC_EXPORTED_TARGETS "aec-static" CACHE BOOL "" FORCE)

set (ZLIB_PACKAGE_NAME "zlib" CACHE STRING "Name of ZLIB package" FORCE)
set (ZLIB_TGZ_NAME "zlib-1.3.1.tar.gz" CACHE STRING "Use HDF5_ZLib from compressed file" FORCE)
set (ZLIB_TGZ_ORIGPATH "https://github.com/madler/zlib/releases/download/v1.3.1" CACHE STRING "Use ZLIB from original location" FORCE)
set (HDF5_USE_ZLIB_STATIC ON CACHE BOOL "Use static zlib library" FORCE)

set (ZLIBNG_PACKAGE_NAME "zlib-ng" CACHE STRING "Name of ZLIBNG package" FORCE)
set (ZLIBNG_TGZ_NAME "2.2.2.tar.gz" CACHE STRING "Use HDF5_ZLib from compressed file" FORCE)
set (ZLIBNG_TGZ_ORIGPATH "https://github.com/zlib-ng/zlib-ng/archive/refs/tags" CACHE STRING "Use ZLIBNG from original location" FORCE)

set (LIBAEC_PACKAGE_NAME "libaec" CACHE STRING "Name of AEC SZIP package" FORCE)
set (LIBAEC_TGZ_NAME "libaec-1.1.3.tar.gz" CACHE STRING "Use SZip AEC from compressed file" FORCE)
set (LIBAEC_TGZ_ORIGPATH "https://github.com/MathisRosenhauer/libaec/releases/download/v1.1.3" CACHE STRING "Use LIBAEC from original location" FORCE)
set (HDF5_USE_LIBAEC_STATIC ON CACHE BOOL "Use static AEC library" FORCE)

FetchContent_MakeAvailable(hdf5)

set (HDF5_DIR "${hdf5_SOURCE_DIR}/cmake" CACHE STRING "" FORCE)

# examples are the tests for plugins
set (H5PL_BUILD_TESTING OFF CACHE BOOL "Enable H5PL testing" FORCE)
set (H5PL_BUILD_EXAMPLES OFF CACHE BOOL "Build H5PL Examples" FORCE)


#preset HDF5 cache vars to this projects libraries instead of searching
set (H5PL_HDF5_HEADER "H5pubconf.h" CACHE STRING "Name of HDF5 header" FORCE)
set (H5PL_HDF5_LINK_LIBS "hdf5-static" CACHE STRING "HDF5 target" FORCE)
set (H5PL_HDF5_INCLUDE_DIRS $<TARGET_PROPERTY:hdf5-static,INCLUDE_DIRECTORIES> CACHE PATH "HDF5 include dirs" FORCE)
set (H5PL_HDF5_DIR ${hdf5_BUILD_DIR} CACHE STRING "HDF5 build folder" FORCE)

set (H5PL_HDF5_DUMP_EXECUTABLE $<TARGET_FILE:h5dump> CACHE STRING "HDF5 h5dump target" FORCE)
set (H5PL_HDF5_REPACK_EXECUTABLE $<TARGET_FILE:h5repack> CACHE STRING "HDF5 h5repack target" FORCE)

set (H5PL_ALLOW_EXTERNAL_SUPPORT "TGZ" CACHE STRING "Allow External Library Building (NO GIT TGZ)" FORCE)

set (H5LZF_RESOURCES_DIR "${CMAKE_SOURCE_DIR}/LZF/config/cmake" CACHE STRING "" FORCE)
set (H5PL_COMP_TGZPATH "http://dist.schmorp.de/liblzf" CACHE STRING "Use LZF from original location" FORCE)
set (LZF_TGZ_NAME "liblzf-3.6.tar.gz" CACHE STRING "Use LZF from compressed file" FORCE)
set (LZF_PACKAGE_NAME "lzf" CACHE STRING "" FORCE)

set (H5LZF_EXPORTED_TARGETS "h5lzf-static" CACHE STRING "" FORCE)

add_subdirectory(LZF)

install(TARGETS "lzf-static" DESTINATION ${CMAKE_INSTALL_PREFIX}/lib)
