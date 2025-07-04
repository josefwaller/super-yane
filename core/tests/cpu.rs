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
    ($name: expr) => { rom_test!{$name, 100 * 5000} }
}

rom_test! {and, 350_000}
rom_test! {asl, 100_000}
rom_test! { bit, 400_000}
rom_test! {bra, 50_000}
rom_test! {cmp, 500_000}
rom_test! {dec, 150_000}
rom_test! {eor, 350_000}
rom_test! {inc, 150_000}
rom_test! {jmp, 50_000}
rom_test! {ldr, 600_000}
rom_test! {lsr, 100_000}
rom_test! {mov, 50_000}
rom_test! {msc, 50_000}
rom_test! {ora, 350_000}
rom_test! {phl}
rom_test! {psr, 50_000}
rom_test! {ret, 50_000}
rom_test! {rol, 100_000}
rom_test! {ror, 100_000}
rom_test! {str, 550_000}
rom_test! {trn, 350_000}
rom_test! {test_basic, 1_400_000}
rom_test! {test_full, 1_950_000}
