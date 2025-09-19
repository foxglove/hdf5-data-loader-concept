pub mod hdf5;
pub mod log;
#[cfg(target_arch = "wasm32")]
pub mod wasm_vfs;

use core::time;
use std::collections::{BTreeMap, BTreeSet};

use hdf5::*;
use smallvec::SmallVec;

type TimestampIndex = BTreeMap<u64, SmallVec<[u64; 4]>>;

#[derive(Debug)]
struct Topic {
    name: String,
    timestamps: TimestampIndex,
}

fn main() -> anyhow::Result<()> {
    init_lzf();

    let loader = use

    let file = Hdf5File::open("/Users/bennett/Downloads/BUV-Nimbus04_L3zm_v01-02-2013m0422t101810.h5")?;

    let datasets = file.get_datasets();

    let mut topics: Vec<Topic> = vec![];

    for dataset in datasets.values() {
        if dataset.name.ends_with(".timestamp") || dataset.name.ends_with(".parameters") {
            continue;
        }

        println!("{:?}", dataset.attrs);

        let mut timestamp_dataset = datasets.get(&format!("{}.timestamp", dataset.name));

        if let Some(Attribute::Vlen(dimensions)) = dataset.attrs.get("DIMENSION_LIST") {
            for (index, dimension) in dimensions.iter().enumerate() {
                let Attribute::Reference(name) = dimension else {
                    continue;
                };

                let Some(dataset) = datasets.get(name) else {
                    continue;
                };

                if !dataset.name.contains("time") {
                    continue;
                }

                timestamp_dataset = Some(dataset);
            }
        }

        let parameters_dataset = datasets.get(&format!("{}.parameters", dataset.name));

        let Some(timestamp_dataset) = timestamp_dataset else {
            println!("skipping {} due to no timestamp", dataset.name);
            continue;
        };

        let mut timestamps: TimestampIndex = Default::default();

        let (timestamp_data, _) = timestamp_dataset.read::<u64>()?;

        assert_eq!(
            timestamp_data.len(),
            timestamp_dataset.dimensions[0] as usize
        );

        for (i, timestamp) in timestamp_data.into_iter().enumerate() {
            let entry = timestamps.entry(timestamp).or_default();
            entry.push(i as u64);
        }

        println!(
            "{ } - {:?} - image:{} - {:?} - {:?}",
            dataset.name,
            dataset.type_,
            dataset.is_image_topic(),
            parameters_dataset.map(|x| &x.attrs),
            dataset.dimensions,
        );

        if (!dataset.is_image_topic()) {
            continue;
        }

        for (_, entry) in timestamps.iter() {
            for i in entry {
                match dataset.type_ {
                    DatasetType::Float => {
                        dataset.read_at_index::<f32>(*i)?;
                    }

                    DatasetType::Integer => {
                        dataset.read_at_index::<i64>(*i)?;
                    }

                    _ => {}
                }
            }
        }

        topics.push(Topic {
            name: dataset.name.clone(),
            timestamps,
        });
    }

    // println!("topics {topics:?}");

    // for dataset in datasets.values() {
    //     let Some(Attribute::Vlen(dims)) = dataset.attrs.get("DIMENSION_LIST") else {
    //         continue;
    //     };

    //     for dim in dims.iter() {
    //         let Attribute::Reference(obj_id) = dim else {
    //             panic!("should be reference");
    //         };

    //         assert!(
    //             datasets.contains_key(obj_id),
    //             "reference should reference something that exists"
    //         );
    //     }
    // }

    // println!("{datasets:#?}");

    Ok(())
}
