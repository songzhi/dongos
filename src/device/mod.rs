pub mod rtc;

pub unsafe fn init_noncore() {
    rtc::init();
}