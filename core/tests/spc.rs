use insta::assert_binary_snapshot;
use paste::paste;
use super_yane::Console;
mod common;

spc_rom_test! {adc, 50_000_000}
spc_rom_test! {and, 50_000_000}
spc_rom_test! {dec, 50_000_000}
spc_rom_test! {eor, 50_000_000}
spc_rom_test! {inc, 50_000_000}
spc_rom_test! {ora, 50_000_000}
spc_rom_test! {sbc, 50_000_000}
spc_rom_test! {full, 50_000_000}
