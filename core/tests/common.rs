#[macro_export]
macro_rules! rom_test {
    ($name: expr, $file: expr, $num_inst: expr) => {
        paste! {
        #[test]
        fn [<test_$name>] () {
            let mut console = Console::with_cartridge(include_bytes!($file));
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
}

#[macro_export]
macro_rules! cpu_rom_test {
    ($name: expr) => {
        cpu_rom_test! {$name, 1_000_000}
    };
    ($name: expr, $num_inst: expr) => {
        rom_test! {$name, concat!("./roms/CPU", stringify!([<$name:upper>]), ".sfc"), $num_inst}
    };
}
#[macro_export]
macro_rules! spc_rom_test {
    ($name: expr, $num_inst: expr) => {
        rom_test! {$name, concat!("./roms/SPC700", stringify!([<$name:upper>]), ".sfc"), $num_inst}
    };
}
