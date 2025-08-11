use insta::assert_binary_snapshot;
use paste::paste;
use super_yane::Console;
mod common;

spc_rom_test! {adc}
spc_rom_test! {and}
spc_rom_test! {dec}
spc_rom_test! {eor}
spc_rom_test! {inc}
spc_rom_test! {ora}
spc_rom_test! {sbc}
spc_rom_test! {full}
