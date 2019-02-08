pub mod rtc;
pub mod pic;
pub mod cpu;

pub unsafe fn init() {
    pic::PICS.lock().initialize();
}

pub unsafe fn init_noncore() {
    rtc::init();
}