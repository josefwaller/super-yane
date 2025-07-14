use insta::assert_binary_snapshot;
use paste::paste;
use super_yane::Console;
mod common;

rom_test! {background_basic, "./roms/8x8BGMap4BPP32x328PAL.sfc", 50_000}
