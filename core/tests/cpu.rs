use insta::assert_binary_snapshot;
use paste::paste;
use super_yane::Console;

macro_rules! rom_test {
    ($name: expr, $num_inst: expr) => {
        paste! {
        #[test]
        fn [<test_$name>] () {
            let mut console = Console::with_cartridge(include_bytes!(concat!("./roms/CPU", stringify!([<$name:upper>]), ".sfc")));
            console.advance_instructions($num_inst);
            assert_binary_snapshot!(
                ".bin",
                console
                    .ppu()
                    .screen_buffer
                    .map(|b| b.to_le_bytes())
                    .into_iter()
                    .flatten()
                    .collect()
            )
        }
        }
    };
    ($name: expr) => { rom_test!{$name, 5000} }
}

rom_test! {bra}
rom_test! {jmp}
rom_test! {ldr, 19 * 5000}
rom_test! {ret}
rom_test! {and, 12 * 5000}
rom_test! {asl, 4 * 5000}
rom_test! { bit, 7 * 5000}
rom_test! {cmp, 17 * 5000}
rom_test! {eor, 11 * 5000}
rom_test! {msc}
rom_test! {ora, 11 * 5000}
rom_test! {psr}
rom_test! {str, 19 * 50000}
