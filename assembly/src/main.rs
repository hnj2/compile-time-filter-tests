#![feature(asm)]

pub mod a {
    pub fn a() {
        unsafe { asm!("nop"); }
        log::info!("A");
        unsafe { asm!("nop"); }
    }

    pub mod a {
        pub fn a() {
            unsafe { asm!("nop"); }
            log::info!("AA");
            unsafe { asm!("nop"); }
        }
    }

    pub mod b {
        pub fn b() {
            unsafe { asm!("nop"); }
            log::info!("AB");
            unsafe { asm!("nop"); }
        }
    }
}

pub mod b {
    pub fn b() {
        unsafe { asm!("nop"); }
        log::info!("B");
        unsafe { asm!("nop"); }
    }
}

fn main() {
    simple_logger::SimpleLogger::new().init().unwrap();
    a::a();
    a::a::a();
    a::b::b();
    b::b();
}
