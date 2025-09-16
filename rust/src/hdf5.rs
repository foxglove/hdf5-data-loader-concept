#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use std::{
    cell::OnceCell,
    collections::BTreeMap,
    ffi::{CStr, CString, c_char, c_void},
    fmt::{Display, write},
    mem::MaybeUninit,
    ptr::null,
    str::FromStr,
    sync::OnceLock,
};

use anyhow::bail;
use hdf5_sys::*;

static FAPL: OnceLock<i64> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
pub fn get_vfs_fapl() -> i64 {
    0
}

const H5Z_LZF: &'static H5Z_class1_t = &H5Z_class1_t {
    id: 32000,
    name: c"lzf".as_ptr(),
    filter: Some(H5Z_lzf_filter),
    set_local: Some(H5Z_lzf_set_local),
    can_apply: None,
};

pub fn init_lzf() {
    unsafe { H5Zregister(H5Z_LZF as *const H5Z_class1_t as *const _) };
}

#[cfg(target_arch = "wasm32")]
pub fn get_vfs_fapl() -> i64 {
    crate::error!("creating vfs thing");

    *FAPL.get_or_init(|| unsafe {
        crate::error!("registering vfs");

        let driver = H5FDregister(&crate::wasm_vfs::WASM_VFS as *const _);

        crate::error!("registered");

        let fapl = H5Pcreate(H5P_CLS_FILE_ACCESS_ID_g);
        H5Pset_driver(fapl, driver, std::ptr::null());

        crate::error!("created driver");

        fapl
    })
}

pub struct Hdf5File {
    handle: i64,
}

#[derive(Debug, Clone)]
pub enum DatasetType {
    Integer,
    Float,
    Time,
    String,
    Bitfield,
    Opaque,
    Compound,
    Reference,
    Enum,
    Vlen,
    Array,
    Complex,
    Nclasses,
}

impl DatasetType {
    fn from_type(val: i32) -> Self {
        match val {
            0 => Self::Integer,
            1 => Self::Float,
            2 => Self::Time,
            3 => Self::String,
            4 => Self::Bitfield,
            5 => Self::Opaque,
            6 => Self::Compound,
            7 => Self::Reference,
            8 => Self::Enum,
            9 => Self::Vlen,
            10 => Self::Array,
            11 => Self::Complex,
            12 => Self::Nclasses,
            _ => panic!("invalid type"),
        }
    }
}

impl Display for DatasetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone)]
pub struct Dataset {
    pub id: i64,
    pub root_id: i64,
    pub type_: DatasetType,
    pub name: String,
    pub original_name: CString,
    pub attrs: BTreeMap<String, Attribute>,
    pub references: Vec<String>,
    pub dimensions: Vec<u64>,
    pub time_dimension: Option<usize>,
}

pub trait ToNativeType: Default + Clone {
    fn native_type() -> i64;
}

impl ToNativeType for u64 {
    fn native_type() -> i64 {
        unsafe { H5T_NATIVE_UINT64_g }
    }
}

impl ToNativeType for f64 {
    fn native_type() -> i64 {
        unsafe { H5T_NATIVE_FLOAT_g }
    }
}

impl Dataset {
    fn is_time(&self) -> bool {
        false
    }

    pub fn is_image_topic(&self) -> bool {
        let is_image_name = ["image", "camera", "video", "depth"]
            .iter()
            .any(|x| self.name.contains(x));

        // time + width + height, or
        // time + width + height + number of cameras
        let is_image_dimensions = self.dimensions.len() == 3 || self.dimensions.len() == 4;

        is_image_name && is_image_dimensions
    }

    pub fn time_dimension_count(&self) -> Option<u64> {
        Some(self.dimensions[self.time_dimension?])
    }

    pub fn read_one<T: ToNativeType>(&self, offset: u64) -> anyhow::Result<Vec<T>> {
        let dset_id = unsafe { H5Dopen2(self.root_id, self.original_name.as_ptr(), 0) };
        let dataspace_id = unsafe { H5Dget_space(dset_id) };
        let ndims = unsafe { H5Sget_simple_extent_ndims(dataspace_id) };

        let mut dims = vec![0_u64; ndims as _];

        unsafe {
            H5Sget_simple_extent_dims(
                dataspace_id,
                dims.as_mut_ptr() as *mut _,
                std::ptr::null_mut(),
            )
        };

        let mut offsets = vec![0; ndims as _];
        offsets[0] = offset;

        let mut counts = dims.clone();
        counts[0] = 1;

        let mut values = vec![T::default(); counts[1..].iter().copied().reduce(|a, b| a * b).unwrap() as usize];

        println!("selecting hyperslab");

        let status = unsafe {
            H5Sselect_hyperslab(
                dataspace_id,
                H5S_seloper_t_H5S_SELECT_SET,
                offsets.as_ptr() as *const _,
                std::ptr::null(),
                counts.as_ptr() as *const _,
                std::ptr::null(),
            )
        };

        assert_eq!(status, 0);

        println!("create namespace");

        let memspace_id =
            unsafe { H5Screate_simple(ndims, counts.as_ptr() as *const _, std::ptr::null()) };

        println!("reading");

        let status = unsafe {
            H5Dread(
                dset_id,
                T::native_type(),
                memspace_id,
                dataspace_id,
                0,
                values.as_mut_ptr() as *mut u64 as *mut _,
            )
        };

        assert_eq!(status, 0);

        println!("closing");

        unsafe {
            H5Sclose(memspace_id);
            H5Sclose(dataspace_id);
            H5Dclose(dset_id);
        };

        Ok(values)
    }

    pub fn read<T: ToNativeType>(&self, offset: u64) -> anyhow::Result<(Vec<T>, Vec<u64>)> {
        let dset_id = unsafe { H5Dopen2(self.root_id, self.original_name.as_ptr(), 0) };
        let dataspace_id = unsafe { H5Dget_space(dset_id) };
        let ndims = unsafe { H5Sget_simple_extent_ndims(dataspace_id) };

        let mut dims = vec![0_u64; ndims as _];

        unsafe {
            H5Sget_simple_extent_dims(
                dataspace_id,
                dims.as_mut_ptr() as *mut _,
                std::ptr::null_mut(),
            )
        };

        let mut values = vec![T::default(); dims.iter().sum::<u64>() as usize];

        let mut offsets = vec![0; ndims as _];
        offsets[0] = offset;

        let status = unsafe {
            H5Sselect_hyperslab(
                dataspace_id,
                H5S_seloper_t_H5S_SELECT_SET,
                offsets.as_ptr() as *const _,
                std::ptr::null(),
                dims.as_ptr() as *const _,
                std::ptr::null(),
            )
        };

        assert_eq!(status, 0);

        let memspace_id =
            unsafe { H5Screate_simple(2, dims.as_ptr() as *const _, std::ptr::null()) };

        let status = unsafe {
            H5Dread(
                dset_id,
                T::native_type(),
                memspace_id,
                dataspace_id,
                0,
                values.as_mut_ptr() as *mut u64 as *mut _,
            )
        };

        assert_eq!(status, 0);

        unsafe {
            H5Sclose(memspace_id);
            H5Sclose(dataspace_id);
            H5Dclose(dset_id);
        };

        Ok((values, dims))
    }
}

#[derive(Default)]
struct ObjectIterateData {
    datasets: BTreeMap<String, Dataset>,
    root_id: hid_t,
}

#[derive(Debug, Clone)]
pub enum Attribute {
    Str(String),
    Vlen(Vec<Attribute>),
    Reference(String),
    Unknown(String),
}

#[derive(Default)]
struct AttrIterateData {
    references: Vec<String>,
    attrs: BTreeMap<String, Attribute>,
    obj: hid_t,
}

unsafe extern "C" fn hfd5_object_attr_visit_callback(
    obj_id: hid_t,
    attr_name: *const ::std::os::raw::c_char,
    ainfo: *const H5A_info_t,
    op_data: *mut ::std::os::raw::c_void,
) -> i32 {
    let data: &mut AttrIterateData = unsafe { &mut *(op_data as *mut AttrIterateData) };

    let attr_id = unsafe { H5Aopen(obj_id, attr_name, 0) };

    let attr_type = unsafe { H5Aget_type(attr_id) };
    let attr_class = unsafe { H5Tget_class(attr_type) };

    let name = unsafe { CStr::from_ptr(attr_name) };
    let name = name.to_string_lossy().to_string();

    match attr_class {
        H5T_class_t_H5T_STRING => {
            let size = unsafe { H5Tget_size(attr_type) };
            let mut str: Vec<u8> = vec![0; size as _];
            unsafe { H5Aread(attr_id, attr_type, &mut str[..] as *mut [u8] as *mut _) };

            let str = String::from_utf8_lossy(&str[..]);

            data.attrs.insert(name, Attribute::Str(str.to_string()));
        }

        H5T_class_t_H5T_VLEN => {
            let base_type = unsafe { H5Tget_super(attr_type) };
            let base_class = unsafe { H5Tget_class(base_type) };
            let space_id = unsafe { H5Aget_space(attr_id) };
            let mut dims = [0_u64; H5S_MAX_RANK as _];
            let ndims =
                H5Sget_simple_extent_dims(space_id, &mut dims as *mut _, std::ptr::null_mut());

            if base_class == H5T_class_t_H5T_REFERENCE {
                let mut values: Vec<hvl_t> = vec![std::mem::zeroed(); dims[0] as usize];
                let ret = unsafe { H5Aread(attr_id, attr_type, values.as_mut_ptr() as *mut _) };

                if ret < 0 {
                    crate::error!("failed to read attribute hvl");
                    return 1;
                }

                let mut attributes: Vec<Attribute> = vec![];

                for value in values.iter() {
                    let ref_type = H5Rget_type(value.p as *const _);

                    if !(ref_type == H5R_type_t_H5R_OBJECT1 || ref_type == H5R_type_t_H5R_OBJECT2) {
                        continue;
                    }

                    let source_obj = H5Rdereference2(obj_id, 0, 0, value.p as *mut _);

                    let mut dataset_name_buf = vec![0_u8; 255];

                    let len = H5Iget_name(source_obj, dataset_name_buf.as_mut_ptr() as *mut _, 255);

                    let dataset_name = String::from_utf8_lossy(&dataset_name_buf[..len as _]);

                    H5Oclose(source_obj);

                    attributes.push(Attribute::Reference(dataset_name.to_string()));
                }

                data.attrs.insert(name, Attribute::Vlen(attributes));
            }
        }

        _ => {
            data.attrs.insert(
                name,
                Attribute::Unknown(format!("{attr_type}/{attr_class}")),
            );
        }
    }

    unsafe { H5Aclose(attr_id) };

    0
}

unsafe extern "C" fn hdf5_object_visit_callback(
    obj: hid_t,
    name: *const ::std::os::raw::c_char,
    info: *const H5O_info1_t,
    op_data: *mut ::std::os::raw::c_void,
) -> i32 {
    let data: &mut ObjectIterateData = unsafe { &mut *(op_data as *mut ObjectIterateData) };
    let info = unsafe { &*info };
    // pub const H5O_type_t_H5O_TYPE_UNKNOWN: H5O_type_t = -1;
    // #[doc = "< Object is a group"]
    // pub const H5O_type_t_H5O_TYPE_GROUP: H5O_type_t = 0;
    // #[doc = "< Object is a dataset"]
    // pub const H5O_type_t_H5O_TYPE_DATASET: H5O_type_t = 1;
    // #[doc = "< Object is a named data type"]
    // pub const H5O_type_t_H5O_TYPE_NAMED_DATATYPE: H5O_type_t = 2;
    // #[doc = "< Object is a map"]
    // pub const H5O_type_t_H5O_TYPE_MAP: H5O_type_t = 3;
    // #[doc = "< Number of different object types (must be last!)"]
    // pub const H5O_type_t_H5O_TYPE_NTYPES: H5O_type_t = 4;

    if info.type_ == H5O_type_t_H5O_TYPE_NAMED_DATATYPE {
        crate::error!("its a datatype");
    }

    if info.type_ == H5O_type_t_H5O_TYPE_DATASET {
        unsafe {
            let dset_id = H5Dopen2(obj, name, 0);
            let space_id = H5Dget_space(dset_id);

            let dataset_type = H5Dget_type(dset_id);
            let dataset_class = H5Tget_class(dataset_type);
            let type_ = DatasetType::from_type(dataset_class);

            let ndims = H5Sget_simple_extent_ndims(space_id);

            let mut dims = [0_u64; H5S_MAX_RANK as _];
            H5Sget_simple_extent_dims(space_id, &mut dims as *mut _, std::ptr::null_mut());

            H5Sclose(space_id);

            let mut attrs = AttrIterateData::default();
            attrs.obj = obj;

            H5Aiterate2(
                dset_id,
                H5_index_t_H5_INDEX_NAME,
                H5_iter_order_t_H5_ITER_INC,
                std::ptr::null_mut(),
                Some(hfd5_object_attr_visit_callback),
                &mut attrs as *mut AttrIterateData as *mut _,
            );

            H5Dclose(dset_id);

            println!("dset: {dset_id}");

            let original_name =
                CString::from_vec_with_nul(CStr::from_ptr(name).to_bytes_with_nul().to_vec())
                    .unwrap();
            let name = original_name.to_string_lossy();

            let prefix = if name.starts_with('/') { "" } else { "/" };
            let name = format!("{prefix}{name}");

            // attrs.attrs.get("DIMENSION_LIST")

            data.datasets.insert(
                name.clone(),
                Dataset {
                    id: dset_id,
                    root_id: data.root_id,
                    type_,
                    name,
                    original_name,
                    dimensions: dims[..ndims as _].to_vec(),
                    attrs: attrs.attrs,
                    references: attrs.references,
                    time_dimension: None,
                },
            );
        }
    }

    0
}

impl Hdf5File {
    pub fn open(file: &str) -> anyhow::Result<Self> {
        let file = CString::from_str(file)?;
        let fapl_id = get_vfs_fapl();

        crate::error!("open file");
        let handle = unsafe { H5Fopen(file.as_ptr(), 0, fapl_id) };

        Ok(Self { handle })
    }

    fn get_group_info(&self) -> H5G_info_t {
        crate::error!("get group info");
        let mut info = unsafe { std::mem::zeroed() };
        unsafe { H5Gget_info(self.handle, &mut info) };
        info
    }

    pub fn get_datasets(&self) -> BTreeMap<String, Dataset> {
        let mut data = ObjectIterateData::default();
        data.root_id = self.handle;

        crate::error!("iterating");

        unsafe {
            H5Ovisit1(
                self.handle,
                H5_index_t_H5_INDEX_NAME,
                H5_iter_order_t_H5_ITER_INC,
                Some(hdf5_object_visit_callback),
                &mut data as *mut ObjectIterateData as *mut _,
            );
        }

        data.datasets
    }

    pub fn links(&self) -> Vec<String> {
        let info = self.get_group_info();
        let mut links = Vec::with_capacity(info.nlinks as _);

        crate::error!("get links");
        for g in 0..info.nlinks {
            unsafe {
                // let oinfo = std::mem::zeroed();
                let mut name = [0 as c_char; 255];

                crate::error!("get link");
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

                let mut info: H5L_info2_t = std::mem::zeroed();

                H5Lget_info2(
                    self.handle,
                    CString::from_str(&s).unwrap().as_ptr(),
                    &mut info,
                    0,
                );

                crate::error!("{s} - {}", info.type_);

                links.push(s);
            }
        }

        links
    }
}
