use std::sync::atomic::{AtomicU32, Ordering};

pub fn unique_tag() -> u32 {
    static COUNT: AtomicU32 = AtomicU32::new(1);

    let tag = COUNT.fetch_add(1, Ordering::Relaxed);
    assert!(
        tag < sys::LUA_UTAG_LIMIT as u32,
        "userdata tag limit exceeded",
    );

    tag
}

pub trait Userdata {
    fn tag() -> u32;
    fn name() -> &'static str;
}
