mod dlopen_stub;

pub mod hdf5;
pub mod log;
pub mod wasm_vfs;

use std::collections::BTreeMap;

use hdf5::*;
use smallvec::SmallVec;

use foxglove_data_loader::{
    DataLoader, Initialization, Message, MessageIterator, MessageIteratorArgs, console,
};

struct Hdf5Iterator {
    topics: Vec<Topic>,
    start: u64,
    end: u64,
    current_time: u64,
    queue: BTreeMap<u64, SmallVec<[Message; 4]>>,
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

            if self.current_time == self.end {
                return None;
            }

            let next_time = (self.current_time + 1024).min(self.end);

            for topic in self.topics.iter() {
                for mm in topic.timestamps.range(self.current_time..next_time) {

                }
            }

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
                let entry = timestamps.entry(timestamp).or_default();
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
        Ok(Hdf5Iterator)
    }
}

foxglove_data_loader::export!(Hdf5Loader);
