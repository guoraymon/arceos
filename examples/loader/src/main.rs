#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0xffff_ffc0_2200_0000;
const HEADER_SIZE: usize = 4;

fn slice_to_usize(slice: &[u8]) -> usize {
    let mut bytes = [0u8; 8];
    bytes[4..8].copy_from_slice(slice);
    usize::from_be_bytes(bytes)
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    let mut apps_start = PLASH_START as *const u8;

    println!("Load payload ...");

    loop {
        let app_size =
            slice_to_usize(unsafe { core::slice::from_raw_parts(apps_start, HEADER_SIZE) });
        if app_size == 0 {
            break;
        }
        apps_start = unsafe { apps_start.add(HEADER_SIZE) };
        let code = unsafe { core::slice::from_raw_parts(apps_start, app_size as usize) };
        println!("content: {:?}: ", code);
        apps_start = unsafe { apps_start.add(app_size) };
    }

    println!("Load payload ok!");
}
