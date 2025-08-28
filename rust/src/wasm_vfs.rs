use hdf5_sys::*;

use std::{
    ffi::{CStr, CString, c_char},
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    mem::MaybeUninit,
    str::FromStr,
    usize,
};

use foxglove_data_loader::{
    console,
    reader::{Reader, open},
};

unsafe extern "C" fn vfs_hdf5_close(h: *mut H5FD_t) -> herr_t {
    println!("closing file");

    0
}

#[repr(C)]
struct WasmFile {
    //
    parent: H5FD_t,
    // fields from here are private to this vfs
    reader: BufReader<Reader>,
    size: u64,
    eoa: u64,
}

unsafe extern "C" fn vfs_hdf5_open(
    name: *const ::std::os::raw::c_char,
    flags: ::std::os::raw::c_uint,
    fapl: hid_t,
    maxaddr: haddr_t,
) -> *mut H5FD_t {
    println!("calling vfs open");

    let path = CStr::from_ptr(name).to_string_lossy();

    console::log(&format!("file name: {path:?}"));

    let reader = open(&*path);
    let size = reader.size();
    let reader = BufReader::new(reader);

    let file = Box::new(WasmFile {
        parent: H5FD_t {
            driver_id: 0,
            cls: (&WASM_VFS) as *const _,
            fileno: 0,
            access_flags: 0,
            feature_flags: 0,
            maxaddr: 0,
            base_addr: 0,
            threshold: 0,
            alignment: 0,
            paged_aggr: false,
        },

        reader,
        size,
        eoa: 0,
    });

    Box::leak(file) as *mut _ as *mut H5FD_t
}

unsafe extern "C" fn vfs_get_eoa(file: *const H5FD_t, type_: H5F_mem_t) -> haddr_t {
    // println!("getting eoa");

    let file: *const WasmFile = file as _;
    (&*file).eoa
}

unsafe extern "C" fn vfs_set_eoa(file: *mut H5FD_t, type_: H5F_mem_t, addr: haddr_t) -> herr_t {
    // println!("setting eoa");

    let file: *mut WasmFile = file as _;
    (&mut *file).eoa = addr as _;
    0
}

unsafe extern "C" fn vfs_get_eof(file: *const H5FD_t, type_: H5F_mem_t) -> haddr_t {
    // println!("getting eof");

    let file: *const WasmFile = file as _;
    (&*file).size
}

unsafe extern "C" fn vfs_write(
    file: *mut H5FD_t,
    type_: H5F_mem_t,
    dxpl: hid_t,
    addr: haddr_t,
    size: usize,
    buffer: *const ::std::os::raw::c_void,
) -> herr_t {
    console::error("tried to call write");

    1
}

unsafe extern "C" fn vfs_read(
    file: *mut H5FD_t,
    type_: H5F_mem_t,
    dxpl: hid_t,
    addr: haddr_t,
    size: usize,
    buffer: *mut ::std::os::raw::c_void,
) -> herr_t {
    // println!("calling read, addr: {addr}, size: {size}");

    let wasm_file: *mut WasmFile = file as _;
    let file = &mut *wasm_file;

    let pos = file.reader.stream_position().unwrap();

    if pos != addr {
        file.reader.seek(SeekFrom::Start(addr)).unwrap();
    }

    let slice = std::slice::from_raw_parts_mut(buffer as *mut u8, size);
    slice.fill(0);

    let mut pos = 0;

    loop {
        let written = (&mut *file)
            .reader
            .read(&mut slice[pos as usize..])
            .unwrap();

        if written == 0 {
            break;
        }

        pos += written;

        if pos >= size {
            break;
        }
    }

    0
}

pub const WASM_VFS: H5FD_class_t = H5FD_class_t {
    name: c"wasm_vfs".as_ptr(),
    truncate: None,
    sb_size: None,

    version: H5FD_CLASS_VERSION,
    value: 123,

    // stole this from somewhere:
    maxaddr: (((1 as haddr_t) << (8 * std::mem::size_of::<usize>() - 1)) - 1),

    fl_map: [
        H5F_mem_t_H5FD_MEM_SUPER, /*default*/
        H5F_mem_t_H5FD_MEM_SUPER, /*super*/
        H5F_mem_t_H5FD_MEM_SUPER, /*btree*/
        H5F_mem_t_H5FD_MEM_SUPER, /*draw*/
        H5F_mem_t_H5FD_MEM_SUPER, /*gheap*/
        H5F_mem_t_H5FD_MEM_SUPER, /*lheap*/
        H5F_mem_t_H5FD_MEM_SUPER, /*ohdr*/
    ],

    fc_degree: H5F_close_degree_t_H5F_CLOSE_WEAK,
    fapl_size: 0,
    dxpl_size: 0,

    terminate: None,
    sb_encode: None,
    sb_decode: None,
    fapl_get: None,
    fapl_copy: None,
    fapl_free: None,
    dxpl_copy: None,
    dxpl_free: None,
    open: Some(vfs_hdf5_open),
    close: Some(vfs_hdf5_close),
    cmp: None,
    query: None,
    get_type_map: None,
    alloc: None,
    free: None,

    get_eoa: Some(vfs_get_eoa),
    set_eoa: Some(vfs_set_eoa),

    get_eof: Some(vfs_get_eof),

    get_handle: None,

    read: Some(vfs_read),
    write: Some(vfs_write),

    read_vector: None,
    write_vector: None,
    read_selection: None,
    write_selection: None,
    flush: None,
    lock: None,
    unlock: None,
    del: None,
    ctl: None,
};
