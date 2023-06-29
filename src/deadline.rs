#[cfg(not(target_arch = "wasm32"))]
pub use normal::Deadline;
#[cfg(target_arch = "wasm32")]
pub use wasm::Deadline;

#[cfg(target_arch = "wasm32")]
mod wasm {

    use js_sys::Date;

    #[derive(Copy, Clone)]
    pub struct Deadline {
        expiry: u64,
    }

    impl Deadline {
        pub fn from_now(millis: u64) -> Deadline {
            Deadline {
                expiry: (Date::now() as u64) + millis,
            }
        }
        pub fn expired(&self) -> bool { Date::now() as u64 >= self.expiry }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod normal {

    use std::time::{Duration, Instant};

    #[derive(Copy, Clone)]
    pub struct Deadline {
        expiry: Instant,
    }

    impl Deadline {
        pub fn from_now(millis: u64) -> Deadline {
            Deadline {
                expiry: Instant::now() + Duration::from_millis(millis),
            }
        }
        pub fn expired(&self) -> bool { Instant::now() >= self.expiry }
    }
}
