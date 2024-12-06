#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0xffff_ffc0_2200_0000;

struct Loader {
    start: *const u8,
}

impl Loader {
    pub fn load(&mut self, size: usize) -> &[u8] {
        let data = unsafe { core::slice::from_raw_parts(self.start, size) };
        self.start = self.start.wrapping_add(size);
        data
    }

    pub fn load_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(self.load(4));
        u32::from_be_bytes(bytes)
    }
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // app running aspace
    // SBI(0x80000000) -> App <- Kernel(0x80200000)
    // va_pa_offset: 0xffff_ffc0_0000_0000
    const RUN_START: usize = 0xffff_ffc0_8010_0000;
    let mut run_code_start: usize = RUN_START;

    let mut loader = Loader {
        start: PLASH_START as *const u8,
    };
    loop {
        println!("Load payload ...");

        let app_size = loader.load_u32() as usize;
        if app_size == 0 {
            break;
        }

        let load_code = loader.load(app_size);
        println!(
            "load code {:?}; address [{:?}]",
            load_code,
            load_code.as_ptr()
        );

        let run_code = unsafe { core::slice::from_raw_parts_mut(run_code_start as *mut u8, app_size) };
        run_code.copy_from_slice(load_code);
        run_code_start += app_size;
        println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        println!("Load payload ok!");
    }

    println!("Execute app ...");

    // execute app
    unsafe {
        core::arch::asm!("
            li      t2, {run_start}
            jalr    t2
            j       .",
            run_start = const RUN_START,
        )
    }
}
