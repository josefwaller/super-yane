pub mod utils {
    macro_rules! hex_fmt {
        () => {
            "0x{:04X}"
        };
    }
    macro_rules! table_row {
        ($label: expr, $field: expr, $format_str: expr) => {
            ($label, text(format!($format_str, $field)).into())
        };
        ($label: expr, $field: ident) => {
            ppu_val!($label, $field, "{}")
        };
    }
    pub(crate) use hex_fmt;
    pub(crate) use table_row;
}
