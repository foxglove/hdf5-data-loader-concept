#![allow(warnings)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

unsafe extern "C" {
    pub fn H5Z_lzf_filter(
        flags: ::std::os::raw::c_uint,
        cd_nelmts: usize,
        cd_values: *const ::std::os::raw::c_uint,
        nbytes: usize,
        buf_size: *mut usize,
        buf: *mut *mut ::std::os::raw::c_void,
    ) -> usize;
}
unsafe extern "C" {
    pub fn H5Z_lzf_set_local(
        dcpl: i64,
        type_: i64,
        space: i64
    ) -> i32;
}
