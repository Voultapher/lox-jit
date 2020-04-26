use std::cmp;
use std::mem;
use std::ops::{Index, IndexMut};
use std::ptr;

use libc;

unsafe fn run_jit_program<T>(user_program: &[u8]) -> T {
    const RWX: i32 = libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE;
    const PAGE_SIZE: usize = 4096;
    // TODO error check

    // Allocate memory and prepare it for execution.
    let size = PAGE_SIZE * cmp::max(user_program.len() / PAGE_SIZE, 1);

    let mut program : *mut libc::c_void = mem::MaybeUninit::uninit().assume_init();
    libc::posix_memalign(&mut program, PAGE_SIZE, size);
    libc::mprotect(program, size, RWX);

    // For sanity and safety, populate with illegal instruction trap.
    libc::memset(program, 0xC4, size);

    // Copy user program into prepared code section.
    ptr::copy_nonoverlapping(user_program.as_ptr(), mem::transmute(program), user_program.len());

    // Jump to prepared code section and execute it.
    let fptr: fn() -> T = mem::transmute(program);
    fptr()
}

fn jit_example() -> i64 {
    let program = vec![
        0x48, 0xC7, 0xC7, 0x16, 0x00, 0x00, 0x00, // mov rdi, 22
        0x48, 0x8D, 0x47, 0x25,                   // lea rax, [rdi + 37]
        0xC3                                      // ret
    ];

    unsafe { run_jit_program(program.as_slice()) }
}

fn main() {
    println!("{}", jit_example());
}
