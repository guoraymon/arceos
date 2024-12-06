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

const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;

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

fn abi_terminate() {
    println!("[ABI:Terminate] Bye, Apps!");
    axstd::process::exit(0);
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

    register_abi(SYS_HELLO, abi_hello as usize);
    register_abi(SYS_PUTCHAR, abi_putchar as usize);
    register_abi(SYS_TERMINATE, abi_terminate as usize);
    
    println!("Execute app ...");
    let arg0: u8 = b'A';

    // execute app
    unsafe {
        core::arch::asm!("
        li      t0, {abi_num}
        slli    t0, t0, 3
        la      t1, {abi_table}
        add     t1, t1, t0
        ld      t1, (t1)
        jalr    t1
        li      t2, {run_start}
        jalr    t2
        j       .",
        run_start = const RUN_START,
        abi_table = sym ABI_TABLE,
        //abi_num = const SYS_HELLO,
        abi_num = const SYS_TERMINATE,
        in("a0") arg0,
        )
    }
}
