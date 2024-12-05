#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0xffff_ffc0_2200_0000;
const HEADER_SIZE: usize = 4;

fn slice_to_usize(slice: &[u8]) -> usize {
    let mut bytes = [0u8; 8];
    bytes[4..8].copy_from_slice(slice);
    usize::from_be_bytes(bytes)
}

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;

static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let load_start = PLASH_START as *const u8;
    let load_size = slice_to_usize(unsafe { core::slice::from_raw_parts(load_start, HEADER_SIZE) });

    println!("Load payload ...");

    let load_start = unsafe { load_start.add(HEADER_SIZE) };
    let load_code = unsafe { core::slice::from_raw_parts(load_start, load_size) };
    println!(
        "load code {:?}; address [{:?}]",
        load_code,
        load_code.as_ptr()
    );

    // app running aspace
    // SBI(0x80000000) -> App <- Kernel(0x80200000)
    // va_pa_offset: 0xffff_ffc0_0000_0000
    const RUN_START: usize = 0xffff_ffc0_8010_0000;

    let run_code = unsafe { core::slice::from_raw_parts_mut(RUN_START as *mut u8, load_size) };
    run_code.copy_from_slice(load_code);
    println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

    println!("Load payload ok!");

    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);

	println!("Execute app ...");
    let arg0: u8 = b'A';

    // execute app
    unsafe { core::arch::asm!("
        la      a7, {abi_table}
        li      t2, {run_start}
        jalr    t2
        j       .",
        run_start = const RUN_START,
        abi_table = sym ABI_TABLE,
    )}
}
