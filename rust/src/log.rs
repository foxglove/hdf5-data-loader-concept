#[macro_export]
macro_rules! error {
    ($($t:tt)*) => {
        #[cfg(target_arch = "wasm32")]
        foxglove_data_loader::console::error(&format!($($t)*));

        #[cfg(not(target_arch = "wasm32"))]
        eprintln!($($t)*);
    }
}
