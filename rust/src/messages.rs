use core::time;

use crate::hdf5::Dataset;
use anyhow::bail;
use foxglove::{Encode, schemas::RawImage};

pub fn serialize_mono16_raw_image(index: u64, dataset: &Dataset) -> anyhow::Result<Vec<u8>> {
    if dataset.dimensions.len() != 3 {
        bail!("invalid dimensions for mono16 image");
    }

    let mut data = dataset.read_at_index::<u16>(index)?;

    for d in data.iter_mut() {
        *d = d.to_le();
    }

    let data = unsafe { data.align_to::<u8>().1.to_vec() };

    let message = RawImage {
        timestamp: None,
        data: data.into(),
        step: (dataset.dimensions[2] * 2) as _,
        width: dataset.dimensions[2] as _,
        height: dataset.dimensions[1] as _,
        encoding: "mono16".to_string(),
        frame_id: Default::default(),
    };

    let mut data = Vec::with_capacity(message.encoded_len().unwrap_or_default());
    message.encode(&mut data)?;

    Ok(data)
}

pub fn serialize_rgb8_raw_image(index: u64, dataset: &Dataset) -> anyhow::Result<Vec<u8>> {
    if dataset.dimensions.len() != 4 {
        bail!("invalid dimensions for rgb8 image");
    }

    let data = dataset.read_at_index::<u8>(index)?;

    let message = RawImage {
        timestamp: None,
        data: data.into(),
        step: (dataset.dimensions[2] * 3) as _,
        width: dataset.dimensions[2] as _,
        height: dataset.dimensions[1] as _,
        encoding: "rgb8".to_string(),
        frame_id: Default::default(),
    };

    let mut data = Vec::with_capacity(message.encoded_len().unwrap_or_default());
    message.encode(&mut data)?;

    Ok(data)
}

#[derive(Encode)]
pub struct RawIntegerDataset {
    dimensions: Vec<u64>,
    dataset: Vec<u64>,
}

pub fn serialize_integer_raw(index: u64, dataset: &Dataset) -> anyhow::Result<Vec<u8>> {
    let mut dimensions = dataset.dimensions.clone();
    dimensions.remove(0);

    let dataset = dataset.read_at_index::<u64>(index)?;

    let message = RawIntegerDataset {
        dimensions,
        dataset,
    };

    let mut data = Vec::with_capacity(message.encoded_len().unwrap_or_default());
    message.encode(&mut data)?;

    Ok(data)
}

#[derive(Encode)]
pub struct RawFloatDataset {
    dimensions: Vec<u64>,
    dataset: Vec<f64>,
}

pub fn serialize_float_raw(index: u64, dataset: &Dataset) -> anyhow::Result<Vec<u8>> {
    let mut dimensions = dataset.dimensions.clone();
    dimensions.remove(0);

    let dataset = dataset.read_at_index::<f64>(index)?;

    let message = RawFloatDataset {
        dimensions,
        dataset,
    };

    let mut data = Vec::with_capacity(message.encoded_len().unwrap_or_default());
    message.encode(&mut data)?;

    Ok(data)
}
