mod dlopen_stub;

pub mod hdf5;
pub mod log;
pub mod wasm_vfs;

use core::time;
use std::{collections::BTreeMap, u64};

use foxglove::{Encode, schemas::RawImage};
use hdf5::*;
use smallvec::SmallVec;

use foxglove_data_loader::{
    DataLoader, Initialization, Message, MessageIterator, MessageIteratorArgs, console,
};

struct Hdf5Iterator {
    topics: Vec<(u16, Topic)>,
    start: u64,
    end: Option<u64>,
    current_time: u64,
    queue: BTreeMap<u64, SmallVec<[Message; 4]>>,
    complete: bool,
}

impl MessageIterator for Hdf5Iterator {
    type Error = String;

    fn next(&mut self) -> Option<Result<Message, Self::Error>> {
        loop {
            while let Some(mut entry) = self.queue.first_entry() {
                let Some(message) = entry.get_mut().pop() else {
                    self.queue.pop_first();
                    continue;
                };

                return Some(Ok(message));
            }

            if self.complete {
                return None;
            }

            if let Some(end) = self.end
                && self.current_time >= end
            {
                self.complete = true;
                continue;
            }

            let next_time = (self.current_time + 1024).min(self.end.unwrap_or(u64::MAX));

            for (channel_id, topic) in self.topics.iter() {
                for (timestamp, indexes) in topic.timestamps.range(self.current_time..next_time) {
                    for index in indexes {
                        if topic.dataset.is_image_topic() {
                            if topic.dataset.dimensions.len() == 3 {
                                let mut data = topic.dataset.read_one::<u16>(*index).unwrap();

                                for d in data.iter_mut() {
                                    *d = d.to_le();
                                }

                                let data = unsafe { data.align_to::<u8>().1.to_vec() };

                                let message = RawImage {
                                    timestamp: None,
                                    data: data.into(),
                                    step: (topic.dataset.dimensions[2] * 2) as _,
                                    width: topic.dataset.dimensions[2] as _,
                                    height: topic.dataset.dimensions[1] as _,
                                    encoding: "mono16".to_string(),
                                    frame_id: Default::default(),
                                };

                                let mut data = vec![];
                                message.encode(&mut data).unwrap();

                                self.queue.entry(*timestamp).or_default().push(Message {
                                    log_time: *timestamp,
                                    publish_time: *timestamp,
                                    channel_id: *channel_id,
                                    data,
                                });
                            }

                            if topic.dataset.dimensions.len() == 4 {
                                let data = topic.dataset.read_one::<u8>(*index).unwrap();

                                let message = RawImage {
                                    timestamp: None,
                                    data: data.into(),
                                    step: (topic.dataset.dimensions[2] * 3) as _,
                                    width: topic.dataset.dimensions[2] as _,
                                    height: topic.dataset.dimensions[1] as _,
                                    encoding: "rgb8".to_string(),
                                    frame_id: Default::default(),
                                };

                                let mut data = vec![];
                                message.encode(&mut data).unwrap();

                                self.queue.entry(*timestamp).or_default().push(Message {
                                    log_time: *timestamp,
                                    publish_time: *timestamp,
                                    channel_id: *channel_id,
                                    data,
                                });
                            }
                        }
                    }
                }
            }

            let Some(next_time) = self
                .topics
                .iter()
                .filter_map(|(_, x)| x.timestamps.range(next_time..).next())
                .map(|(timestamp, _)| *timestamp)
                .min()
            else {
                self.complete = true;
                continue;
            };

            self.current_time = next_time;

            continue;
        }
    }
}

type TimestampIndex = BTreeMap<u64, SmallVec<[u64; 4]>>;

#[derive(Debug, Clone)]
struct Topic {
    name: String,
    dataset: Dataset,
    message_count: u64,
    timestamps: TimestampIndex,
}

struct Hdf5Loader {
    path: String,
    file: Option<Hdf5File>,
    topics: Vec<Topic>,
}

impl DataLoader for Hdf5Loader {
    type MessageIterator = Hdf5Iterator;
    type Error = anyhow::Error;

    fn new(args: foxglove_data_loader::DataLoaderArgs) -> Self {
        let path = args.paths.first().unwrap();
        Self {
            path: path.clone(),
            file: None,
            topics: vec![],
        }
    }

    fn initialize(&mut self) -> Result<foxglove_data_loader::Initialization, Self::Error> {
        console::log("creating loader");

        init_lzf();

        let file = self.path.clone();

        let mut init = Initialization::builder();

        console::error("opening a file");

        let file = Hdf5File::open(&file)?;
        let datasets = file.get_datasets();

        for dataset in datasets.values() {
            if dataset.name.ends_with(".timestamp") || dataset.name.ends_with(".parameters") {
                continue;
            }

            let timestamp_dataset = datasets.get(&format!("{}.timestamp", dataset.name));
            let parameters_dataset = datasets.get(&format!("{}.parameters", dataset.name));

            let Some(timestamp_dataset) = timestamp_dataset else {
                println!("skipping {} due to no timestamp", dataset.name);
                continue;
            };

            let mut timestamps: TimestampIndex = Default::default();

            let (timestamp_data, _) = timestamp_dataset.read::<u64>(0)?;

            let mut message_count = 0;

            for (i, timestamp) in timestamp_data.into_iter().enumerate() {
                let entry = timestamps.entry(timestamp * 1000 * 1000).or_default();
                entry.push(i as u64);
                message_count += 1;
            }

            self.topics.push(Topic {
                name: dataset.name.clone(),
                message_count,
                dataset: dataset.clone(),
                timestamps,
            });
        }

        let min = self
            .topics
            .iter()
            .flat_map(|x| x.timestamps.first_key_value())
            .map(|(timestamp, _)| timestamp)
            .min();

        let max = self
            .topics
            .iter()
            .flat_map(|x| x.timestamps.last_key_value())
            .map(|(timestamp, _)| timestamp)
            .max();

        if let Some(min) = min {
            init = init.start_time(*min);
        }

        if let Some(max) = max {
            init = init.end_time(*max);
        }

        for (index, topic) in self.topics.iter().enumerate() {
            if topic.dataset.is_image_topic() {
                init.add_encode::<foxglove::schemas::RawImage>()?
                    .add_channel_with_id(index as _, &topic.name)
                    .unwrap()
                    .message_count(topic.timestamps.len() as _);
            } else {
                init.add_channel_with_id(index as _, &topic.name)
                    .expect("wont exist")
                    .message_count(topic.timestamps.len() as _)
                    .message_encoding("json");
            }
        }

        self.file = Some(file);

        Ok(init.build())
    }

    fn create_iter(
        &mut self,
        args: MessageIteratorArgs,
    ) -> Result<Self::MessageIterator, Self::Error> {
        console::error(&format!("{:?}", args.channels));

        Ok(Hdf5Iterator {
            start: args.start_time.unwrap_or_default(),
            current_time: args.start_time.unwrap_or_default(),
            end: args.end_time,
            queue: Default::default(),
            topics: args
                .channels
                .into_iter()
                .map(|i| (i, self.topics[i as usize].clone()))
                .collect(),
            complete: false,
        })
    }
}

foxglove_data_loader::export!(Hdf5Loader);
