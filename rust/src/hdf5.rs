#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use std::{
    cell::OnceCell,
    ffi::{c_char, CString},
    mem::MaybeUninit,
    str::FromStr, sync::OnceLock,
};

use hdf5_sys::*;

use crate::wasm_vfs;
use foxglove_data_loader::console;

static FAPL: OnceLock<i64> = OnceLock::new();

pub fn get_vfs_fapl() -> i64 {
    console::error("creating vfs thing");

    *FAPL.get_or_init(|| unsafe {
        console::error("registering vfs");

        let driver = H5FDregister(&wasm_vfs::WASM_VFS as *const _);

        console::error("registered");

        let fapl = H5Pcreate(H5P_CLS_FILE_ACCESS_ID_g);
        H5Pset_driver(fapl, driver, std::ptr::null());

        console::error("created driver");

        fapl
    })
}

pub struct Hdf5File {
    handle: i64,
}

impl Hdf5File {
    pub fn open(file: &str) -> anyhow::Result<Self> {
        let file = CString::from_str(file)?;
        let fapl_id = get_vfs_fapl();

        console::error("open file");
        let handle = unsafe { H5Fopen(file.as_ptr(), H5F_ACC_RDWR, fapl_id) };

        Ok(Self { handle })
    }

    fn get_group_info(&self) -> H5G_info_t {
        console::error("get group info");
        let mut info = unsafe { std::mem::zeroed() };
        unsafe { H5Gget_info(self.handle, &mut info) };
        info
    }

    pub fn links(&self) -> Vec<String> {
        let info = self.get_group_info();
        let mut links = Vec::with_capacity(info.nlinks as _);

        console::error("get links");
        for g in 0..info.nlinks {
            unsafe {
                // let oinfo = std::mem::zeroed();
                let mut name = [0 as c_char; 255];

        console::error("get link");
                let len = H5Lget_name_by_idx(
                    self.handle,
                    c".".as_ptr(),
                    H5_index_t_H5_INDEX_NAME,
                    H5_iter_order_t_H5_ITER_INC,
                    g as _,
                    &mut name as *mut _,
                    255,
                    0,
                );

                let mut s = String::with_capacity(len as _);

                for c in name[..len as usize].iter() {
                    s.push(*c as u8 as char);
                }

                links.push(s);
            }
        }

        links
    }
}
