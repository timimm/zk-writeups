#![cfg_attr(feature = "guest", no_std)]
#![no_main]

#[jolt::provable]
fn two_plus_two() -> u16 {
    let mut n: u16 = 2;

    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    unsafe {
        core::arch::asm!(
            "li {n}, 2",
            "add {n}, {n}, {n}",
            n = inout(reg) n,
        );
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!(
            "mov {n}, 2",
            "add {n}, {n}, {n}",
            n = inout(reg) n,
        );
    }
    n
}
