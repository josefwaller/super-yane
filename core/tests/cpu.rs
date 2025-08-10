use insta::assert_binary_snapshot;
use paste::paste;
use super_yane::Console;
mod common;

// use common::cpu_rom_test;
cpu_rom_test! {adc}
cpu_rom_test! {and}
cpu_rom_test! {asl}
cpu_rom_test! {bit}
cpu_rom_test! {bra}
cpu_rom_test! {cmp}
cpu_rom_test! {dec}
cpu_rom_test! {eor}
cpu_rom_test! {inc}
cpu_rom_test! {jmp}
cpu_rom_test! {ldr}
cpu_rom_test! {lsr}
cpu_rom_test! {mov}
cpu_rom_test! {msc}
cpu_rom_test! {ora}
cpu_rom_test! {phl}
cpu_rom_test! {psr}
cpu_rom_test! {ret}
cpu_rom_test! {rol}
cpu_rom_test! {ror}
cpu_rom_test! {sbc}
cpu_rom_test! {str}
cpu_rom_test! {trn}
cpu_rom_test! {test_basic, 2_400_000}
cpu_rom_test! {test_full, 2_950_000}
