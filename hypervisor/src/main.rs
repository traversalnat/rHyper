#![no_std]
#![no_main]
#![feature(asm_const, naked_functions)]
#![feature(panic_info_message, alloc_error_handler)]
#![allow(dead_code)]

#[macro_use]
extern crate log;

extern crate alloc;
#[macro_use]
mod logging;

mod arch;
mod config;
mod device;
mod hv;
mod mm;
mod platform;
mod timer;

mod utils;

#[cfg(not(test))]
mod lang_items;

use core::sync::atomic::{AtomicBool, Ordering};

use crate::{
    config::{GUEST_ENTRIES, PSCI_CONTEXT},
    hv::gconfig::{GUEST_DTB, GUEST_DTB_ADDR, GUEST_ENTRY, GUEST_INITRAMFS, GUEST_INITRAMFS_START},
    platform::mp::start_secondary_cpus,
};

static INIT_OK: AtomicBool = AtomicBool::new(false);

const LOGO: &str = r"

    RRRRRR  VV     VV MM    MM
    RR   RR VV     VV MMM  MMM
    RRRRRR   VV   VV  MM MM MM
    RR  RR    VV VV   MM    MM
    RR   RR    VVV    MM    MM
     ___    ____    ___    ___
    |__ \  / __ \  |__ \  |__ \
    __/ / / / / /  __/ /  __/ /
   / __/ / /_/ /  / __/  / __/
  /____/ \____/  /____/ /____/
";

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn boot_stack();
    fn boot_stack_top();
    fn ekernel();
}

fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0)
    }
}

pub fn init_ok() -> bool {
    INIT_OK.load(Ordering::SeqCst)
}

#[no_mangle]
fn rust_main(cpu_id: usize, dtb_addr: usize) {
    clear_bss();
    arch::init();
    device::init_early();
    println!("{}", LOGO);
    println!("primary cpu id: {}.", cpu_id);
    println!(
        "boot stack: 0x{:x}, boot stack top: 0x{:x}",
        boot_stack as usize, boot_stack_top as usize
    );
    println!(
        "\
        arch = {}\n\
        build_mode = {}\n\
        log_level = {}\n\
        ",
        option_env!("ARCH").unwrap_or(""),
        option_env!("MODE").unwrap_or(""),
        option_env!("LOG").unwrap_or(""),
    );

    mm::init_heap_early();
    logging::init();
    info!("Logging is enabled.");

    mm::init();
    device::init();
    INIT_OK.store(true, Ordering::SeqCst);
    info!("Initialization completed.\n");
    info!(
        "GUEST DTB 0x{:x} to HOST 0x{:x}",
        GUEST_DTB_ADDR,
        GUEST_DTB.as_ptr() as usize
    );
    info!(
        "GUEST INITRAMFS 0x{:x} to HOST 0x{:x}",
        GUEST_INITRAMFS_START,
        GUEST_INITRAMFS.as_ptr() as usize
    );
    start_secondary_cpus(cpu_id);
    unsafe {
        hv::run(cpu_id, GUEST_ENTRY, GUEST_DTB_ADDR);
    }
    // arch::instructions::wait_for_ints();
}

#[no_mangle]
fn rust_main_secondary(cpu_id: usize) {
    // todo
    arch::init();
    device::init();
    info!("Hello World from cpu {}", cpu_id);
    // Safety: Modify to usize is atomic; there is most one writer at the same time.
    unsafe {
        while GUEST_ENTRIES[cpu_id] == 0 {
            trace!("secondary cpu {} waiting", cpu_id);
        }
        info!(
            "secondary cpu {} will run a vcpu with entry 0x{:x}",
            cpu_id, GUEST_ENTRIES[cpu_id]
        );
        hv::run(cpu_id, GUEST_ENTRIES[cpu_id], PSCI_CONTEXT[cpu_id]);
    }
    // console_putchar('z' as u8);
    // console_putchar('b' as u8);
    // console_putchar('d' as u8);
    // info!("CPU {} initialized.", cpu_id);
    // info!("CPU {} initialized.", cpu_id);
    // wait_for_ints();
}

// fn test_output(val: &str) {
//     static LOCK: Mutex<()> = Mutex::new(());
//     LOCK.lock();
//     for s in val.chars() {
//         console_putchar(s as u8);
//     }
// }
