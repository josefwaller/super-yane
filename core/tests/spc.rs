use insta::assert_binary_snapshot;
use paste::paste;
use super_yane::Console;
mod common;

spc_rom_test! {adc, 1_000_000}
spc_rom_test! {and, 1_100_000}
spc_rom_test! {dec, 700_000}
spc_rom_test! {eor, 1_000_000}
spc_rom_test! {inc, 700_000}
spc_rom_test! {ora, 1_100_000}
spc_rom_test! {sbc, 1_100_000}
spc_rom_test! {full, 12_500_000}
