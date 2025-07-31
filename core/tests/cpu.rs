use insta::assert_binary_snapshot;
use paste::paste;
use super_yane::Console;
mod common;

// use common::cpu_rom_test;
cpu_rom_test! {adc, 500_000}
cpu_rom_test! {and, 350_000}
cpu_rom_test! {asl, 100_000}
cpu_rom_test! {bit, 400_000}
cpu_rom_test! {bra, 50_000}
cpu_rom_test! {cmp, 500_000}
cpu_rom_test! {dec, 150_000}
cpu_rom_test! {eor, 350_000}
cpu_rom_test! {inc, 150_000}
cpu_rom_test! {jmp, 50_000}
cpu_rom_test! {ldr, 600_000}
cpu_rom_test! {lsr, 100_000}
cpu_rom_test! {mov, 50_000}
cpu_rom_test! {msc, 50_000}
cpu_rom_test! {ora, 350_000}
cpu_rom_test! {phl, 400_000}
cpu_rom_test! {psr, 50_000}
cpu_rom_test! {ret, 50_000}
cpu_rom_test! {rol, 100_000}
cpu_rom_test! {ror, 100_000}
cpu_rom_test! {sbc, 400_000}
cpu_rom_test! {str, 550_000}
cpu_rom_test! {trn, 350_000}
cpu_rom_test! {test_basic, 1_400_000}
cpu_rom_test! {test_full, 1_950_000}
