mod dlopen_stub;

pub mod hdf5;
pub mod log;
pub mod messages;
pub mod wasm_vfs;

use std::{borrow::Cow, collections::BTreeMap};

use foxglove::{Encode, Schema, schemas::RawImage};
use hdf5::*;
use messages::{
    RawFloatDataset, RawIntegerDataset, serialize_float_raw, serialize_integer_raw,
    serialize_mono16_raw_image, serialize_rgb8_raw_image,
};
use smallvec::SmallVec;

use foxglove_data_loader::{
    BackfillArgs, DataLoader, Initialization, Message, MessageIterator, MessageIteratorArgs,
    Problem, console,
};

struct Hdf5Iterator {
    topics: Vec<(u16, Topic)>,
    start: u64,
    end: Option<u64>,
    current_time: u64,
    queue: BTreeMap<u64, SmallVec<[Message; 4]>>,
    complete: bool,
}

macro_rules! handle {
    ($e:expr) => {
        match $e {
            Err(e) => {
                return Some(Err(e.into()));
            }
            Ok(x) => x,
        }
    };
}

impl MessageIterator for Hdf5Iterator {
    type Error = anyhow::Error;

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
                for (timestamp, _) in topic.timestamps.range(self.current_time..next_time) {
                    if let Some(messages) = topic.messages_at(*channel_id, *timestamp) {
                        self.queue
                            .entry(*timestamp)
                            .or_default()
                            .extend(handle!(messages));
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

trait SerializeMessage: std::fmt::Debug {
    fn to_message(&self, index: u64, dataset: &Dataset) -> Vec<u8>;
}

#[derive(Debug, Clone)]
struct Topic {
    dataset: Dataset,
    timestamps: TimestampIndex,
    serialize_message: fn(u64, &Dataset) -> anyhow::Result<Vec<u8>>,
}

impl Topic {
    fn messages_at(
        &self,
        channel_id: u16,
        timestamp: u64,
    ) -> Option<anyhow::Result<SmallVec<[Message; 4]>>> {
        let indexes = self.timestamps.get(&timestamp)?;

        let mut out = SmallVec::<[Message; 4]>::new();

        for index in indexes {
            let data = handle!((self.serialize_message)(*index, &self.dataset));
            out.push(Message {
                channel_id,
                log_time: timestamp,
                publish_time: timestamp,
                data,
            });
        }

        Some(Ok(out))
    }
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

        let mut channel_id: u16 = 0;

        for dataset in datasets.values() {
            if dataset.name.contains(".timestamp") {
                continue;
            }

            let mut timestamp_dataset = datasets.get(&format!("{}.timestamp", dataset.name));

            error!("ATTRS: {:?}", dataset.attrs);

            if let Some(Attribute::Vlen(dimensions)) = dataset.attrs.get("DIMENSION_LIST") {
                for (index, dimension) in dimensions.iter().enumerate() {
                    let Attribute::Reference(name) = dimension else {
                        continue;
                    };

                    let Some(dataset) = datasets.get(name) else {
                        continue;
                    };

                    if dataset.name.contains("time") || dataset.name.contains("Time") {
                        timestamp_dataset = Some(dataset);
                    }

                }
            }

            let Some(timestamp_dataset) = timestamp_dataset else {
                init = init.add_problem(Problem::warn(format!("Missing timestamps for {}", dataset.name))
                    .tip(format!("Ensure that the dataset {}.timestamp exists or specify a time dataset with DIMENSION_LIST attribute.", dataset.name)));
                continue;
            };

            if dataset.time_dimension.is_none() {
                init = init.add_problem(Problem::warn(format!("Unknown timestamp dimension for {}", dataset.name))
                    .tip("Unable to determine what dimension of the dataset should be used for time."));
                continue;
            }

            let mut topic_name: Option<String> = None;
            let mut topic_schema: Option<Schema> = None;
            let mut is_ros_2: Option<bool> = None;

            let mut timestamps: TimestampIndex = Default::default();

            let (timestamp_data, _) = timestamp_dataset.read::<u64>()?;

            let mut message_count = 0;

            for (i, timestamp) in timestamp_data.into_iter().enumerate() {
                let entry = timestamps.entry(timestamp * 1000 * 1000).or_default();
                entry.push(i as u64);
                message_count += 1;
            }

            match dataset.type_ {
                DatasetType::Integer => {
                    self.topics.push(Topic {
                        dataset: dataset.clone(),
                        serialize_message: serialize_integer_raw,
                        timestamps: timestamps.clone(),
                    });

                    init.add_encode::<RawIntegerDataset>()?
                        .add_channel_with_id(channel_id, &dataset.name)
                        .expect("not in use")
                        .message_count(message_count);

                    channel_id += 1;

                    if dataset.is_image_topic() {
                        if dataset.dimensions.len() == 3 {
                            self.topics.push(Topic {
                                dataset: dataset.clone(),
                                serialize_message: serialize_mono16_raw_image,
                                timestamps: timestamps.clone(),
                            });

                            init.add_encode::<RawImage>()?
                                .add_channel_with_id(
                                    channel_id,
                                    &format!("{}/as_image", dataset.name),
                                )
                                .expect("not in use")
                                .message_count(message_count);

                            channel_id += 1;
                        }

                        if dataset.dimensions.len() == 4 {
                            self.topics.push(Topic {
                                dataset: dataset.clone(),
                                serialize_message: serialize_rgb8_raw_image,
                                timestamps,
                            });

                            init.add_encode::<RawImage>()?
                                .add_channel_with_id(
                                    channel_id,
                                    &format!("{}/as_image", dataset.name),
                                )
                                .expect("not in use")
                                .message_count(message_count);

                            channel_id += 1;
                        }
                    }
                }
                DatasetType::Float => {
                    self.topics.push(Topic {
                        dataset: dataset.clone(),
                        serialize_message: serialize_float_raw,
                        timestamps,
                    });

                    init.add_encode::<RawFloatDataset>()?
                        .add_channel_with_id(channel_id, &dataset.name)
                        .expect("not in use")
                        .message_count(message_count);

                    channel_id += 1;
                }
                t => {
                    init = init.add_problem(
                        Problem::warn(format!("Unsupported format for {}", dataset.name))
                            .tip(format!("The dataset type of {t:?} is not supported.")),
                    );
                    continue;
                }
            }
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

    fn get_backfill(&mut self, args: BackfillArgs) -> Result<Vec<Message>, Self::Error> {
        let mut out = vec![];

        for channel_id in args.channels {
            let topic = &self.topics[channel_id as usize];

            let Some((timestamp, _)) = topic.timestamps.range(..args.time).next_back() else {
                continue;
            };

            let Some(messages) = topic.messages_at(channel_id, *timestamp) else {
                continue;
            };

            out.extend(messages?);
        }

        Ok(out)
    }
}

foxglove_data_loader::export!(Hdf5Loader);
