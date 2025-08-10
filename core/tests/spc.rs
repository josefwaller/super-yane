use insta::assert_binary_snapshot;
use paste::paste;
use super_yane::Console;
mod common;

spc_rom_test! {adc, 10_000_000}
spc_rom_test! {and, 10_100_000}
spc_rom_test! {dec, 7_000_000}
spc_rom_test! {eor, 10_000_000}
spc_rom_test! {inc, 70_00_000}
spc_rom_test! {ora, 10_100_000}
spc_rom_test! {sbc, 10_100_000}
spc_rom_test! {full, 12_500_000}
