#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0xffff_ffc0_2200_0000;
const HEADER_SIZE: usize = 4;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let image_start = PLASH_START as *const u8;
    let header = unsafe { core::slice::from_raw_parts(image_start, HEADER_SIZE) };
    let apps_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);

    println!("Load payload ...");

    let apps_start = unsafe { image_start.add(HEADER_SIZE) };
    let code = unsafe { core::slice::from_raw_parts(apps_start, apps_size as usize) };
    println!("content: {:?}: ", code);

    println!("Load payload ok!");
}
