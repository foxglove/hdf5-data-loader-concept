#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use hdf5_sys::*;

mod dlopen_stub;
mod wasm_vfs;

use std::{
    ffi::{CStr, CString, c_char},
    fs::File,
    io::{Read, Seek, SeekFrom},
    mem::MaybeUninit,
    str::FromStr,
    usize,
};

use foxglove_data_loader::{
    DataLoader, Initialization, Message, MessageIterator, MessageIteratorArgs, console,
};

struct Hdf5Iterator;

impl MessageIterator for Hdf5Iterator {
    type Error = String;

    fn next(&mut self) -> Option<Result<Message, Self::Error>> {
        None
    }
}

struct Hdf5Loader {
    path: String,
}

impl DataLoader for Hdf5Loader {
    type MessageIterator = Hdf5Iterator;
    type Error = String;

    fn new(args: foxglove_data_loader::DataLoaderArgs) -> Self {
        let path = args.paths.get(0).unwrap();
        Self { path: path.clone() }
    }

    fn initialize(&mut self) -> Result<foxglove_data_loader::Initialization, Self::Error> {
        console::log("creating loader");

        let file = self.path.clone();

        let mut init = Initialization::builder();

        unsafe {
            console::log("registering vfs");

            let driver = H5FDregister(&wasm_vfs::WASM_VFS as *const _);

            console::log("registered");

            let fapl = H5Pcreate(H5P_CLS_FILE_ACCESS_ID_g);
            H5Pset_driver(fapl, driver, std::ptr::null());

            console::log("set driver");

            let file = CString::from_str(&file).unwrap();

            console::log("opening file");

            let file = H5Fopen(file.as_ptr(), 0, fapl);

            console::log("getting info");

            let mut ginfo: MaybeUninit<H5G_info_t> = MaybeUninit::uninit();

            H5Gget_info(file, ginfo.assume_init_mut() as *mut _);

            let ginfo = ginfo.assume_init();

            console::log("iterating topics");

            for g in 0..ginfo.nlinks {
                let mut oinfo: MaybeUninit<H5O_info1_t> = MaybeUninit::uninit();

                let mut name = [0 as c_char; 255];

                let len = H5Lget_name_by_idx(
                    file,
                    c".".as_ptr(),
                    H5_index_t_H5_INDEX_NAME,
                    H5_iter_order_t_H5_ITER_INC,
                    g as _,
                    &mut name as *mut _,
                    255,
                    0,
                );

                let mut s = String::new();

                for c in name[..len as usize].iter() {
                    s.push(*c as u8 as char);
                }

                init.add_channel(&s).message_encoding("json");
            }

            // println!("g: {ginfo:?}");

            // let file = H5Dopen1(file, c"/".as_ptr());
        }

        Ok(init.build())
    }

    fn create_iter(&mut self, args: MessageIteratorArgs) -> Result<Self::MessageIterator, String> {
        Ok(Hdf5Iterator)
    }
}

foxglove_data_loader::export!(Hdf5Loader);
