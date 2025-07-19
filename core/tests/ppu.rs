use insta::assert_binary_snapshot;
use paste::paste;
use super_yane::Console;
mod common;

rom_test! {background_basic_4bpp, "./roms/8x8BGMap4BPP32x328PAL.sfc", 50_000}

rom_test! {background_basic_2bpp_1, "./roms/8x8BG1Map2BPP32x328PAL.sfc", 50_000}
rom_test! {background_basic_2bpp_2, "./roms/8x8BG2Map2BPP32x328PAL.sfc", 50_000}
rom_test! {background_basic_2bpp_3, "./roms/8x8BG3Map2BPP32x328PAL.sfc", 50_000}
rom_test! {background_basic_2bpp_4, "./roms/8x8BG4Map2BPP32x328PAL.sfc", 50_000}
