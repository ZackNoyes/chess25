
#[cfg(not(target_arch = "wasm32"))]
use std::{collections::HashMap, time::Instant};

#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[derive(Clone)]
pub struct Logger {
    /// 10 is everything and 0 is nothing
    level: u8,

    #[cfg(not(target_arch = "wasm32"))]
    start_times: HashMap<String, Instant>
}

impl Logger {

    pub fn new(level: u8) -> Self {
        Logger {
            level,
            #[cfg(not(target_arch = "wasm32"))]
            start_times: HashMap::new()
        }
    }

    pub fn log(&self, level: u8, msg: &str) {
        if level <= self.level {

            #[cfg(not(target_arch = "wasm32"))]
            println!("{}", msg);

            #[cfg(target_arch = "wasm32")]
            console::log_1(&msg.into());

        }
    }

    pub fn log_lazy(&self, level: u8, msg: impl FnOnce() -> String) {
        if level <= self.level {
            self.log(level, &msg());
        }
    }

    pub fn log_lazy_arr(&self, level: u8, _msg: impl FnOnce() -> js_sys::Array) {

        if level <= self.level {

            #[cfg(not(target_arch = "wasm32"))]
            println!("Tried to log a JS array");

            #[cfg(target_arch = "wasm32")]
            console::log_1(&_msg());
        }

    }

    pub fn time_start(&mut self, _level: u8, name: &str) {

        #[cfg(not(target_arch = "wasm32"))]
        self.start_times.insert(name.to_string(), Instant::now());

        #[cfg(target_arch = "wasm32")]
        if _level <= self.level { console::time_with_label(name); }
    }

    pub fn time_end(&mut self, level: u8, name: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(start) = self.start_times.remove(name) {
            let elapsed = start.elapsed();
            self.log(level, &format!(
                "{}: {}.{:03}",
                name, elapsed.as_secs(), elapsed.subsec_millis()
            ));
        } else { panic!("end_time called for non-existing timing string") }

        #[cfg(target_arch = "wasm32")]
        if level <= self.level { console::time_end_with_label(name); }
    }
}