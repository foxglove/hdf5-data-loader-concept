#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use hdf5_sys::*;

mod wasm_vfs;

const WIT: &str = include_str!("../wit/loader.wit");

wit_bindgen::generate!({
    world: "host",
    inline: WIT
});

use std::{
    ffi::{CStr, CString, c_char},
    fs::File,
    io::{Read, Seek, SeekFrom},
    mem::MaybeUninit,
    str::FromStr,
    usize,
};

use exports::foxglove::loader::loader::{
    BackfillArgs, Channel, DataLoader, Guest, GuestDataLoader, GuestMessageIterator, Message,
    MessageIterator, MessageIteratorArgs, TimeRange,
};
use foxglove::loader::{
    console,
    reader::{Reader, open},
};

// use hdf5_bindings::{
//     H5Dopen1, H5Dopen2, H5F_mem_t, H5F_mem_t_H5FD_MEM_SUPER, H5FD_CLASS_VERSION, H5FD_class_t,
//     H5FD_t, H5FDregister, H5Fopen, H5Oopen, H5P_CLS_FILE_ACCESS_ID_g, H5Pcreate, H5Pset_driver,
//     H5Pset_fapl_family, haddr_t, herr_t, hid_t,
// };
//
// mod hdf5_bindings;

struct Hdf5Iterator;

impl GuestMessageIterator for Hdf5Iterator {
    fn next(&self) -> Option<Result<Message, String>> {
        None
    }
}

struct Loader {
    channels: Vec<String>,
}

impl GuestDataLoader for Loader {
    fn channels(&self) -> Result<Vec<Channel>, String> {
        Ok(self
            .channels
            .iter()
            .enumerate()
            .map(|(i, x)| Channel {
                id: i as _,
                topic_name: x.to_string(),
                schema_name: "".to_string(),
                schema_data: vec![],
                schema_encoding: "".to_string(),
                message_encoding: "json".to_string(),
            })
            .collect::<Vec<_>>())
    }

    fn time_range(&self) -> Result<TimeRange, String> {
        Ok(TimeRange {
            start_nanos: 0,
            end_nanos: 0,
        })
    }

    fn get_backfill(&self, args: BackfillArgs) -> Result<Vec<Message>, String> {
        Ok(vec![])
    }

    fn create_iter(&self, args: MessageIteratorArgs) -> Result<MessageIterator, String> {
        Ok(MessageIterator::new(Hdf5Iterator))
    }
}

impl Guest for Loader {
    type DataLoader = Loader;
    type MessageIterator = Hdf5Iterator;

    fn create(mut input: Vec<String>) -> Result<DataLoader, String> {
        console::log("creating loader");

        if input.len() != 1 {
            return Err(format!("got {} files but wanted 1", input.len()));
        }

        let file = input.remove(0);
        let mut channels = vec![];

        unsafe {
            console::log("registering vfs");

            let driver = H5FDregister(&WASM_VFS as *const _);

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

                channels.push(s);
            }

            // println!("g: {ginfo:?}");

            // let file = H5Dopen1(file, c"/".as_ptr());
        }

        Ok(DataLoader::new(Loader { channels }))
    }
}

export!(Loader);
