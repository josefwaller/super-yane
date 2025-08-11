#[macro_export]
macro_rules! rom_test {
    ($name: expr, $file: expr) => {
        rom_test! {$name, $file, 60 * 1}
    };
    ($name: expr, $file: expr, $num_frames: expr) => {
        paste! {
        #[test]
        fn [<test_$name>] () {
            let mut c = Console::with_cartridge(include_bytes!($file));
            (0..$num_frames).for_each(|_| {
                loop {
                    let v = c.ppu().is_in_vblank();
                    c.advance_instructions(1);
                    if !v && c.ppu().is_in_vblank() {
                        break;
                    }
                }
            });
            assert_binary_snapshot!(
                ".bin",
                c
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
        cpu_rom_test! {$name, 60 * 1}
    };
    ($name: expr, $num_frame: expr) => {
        rom_test! {$name, concat!("./roms/CPU", stringify!([<$name:upper>]), ".sfc"), $num_frame}
    };
}
#[macro_export]
macro_rules! spc_rom_test {
    ($name: expr) => {
        spc_rom_test! {$name, 60 * 30}
    };
    ($name: expr, $num_frame: expr) => {
        rom_test! {$name, concat!("./roms/SPC700", stringify!([<$name:upper>]), ".sfc"), $num_frame}
    };
}
