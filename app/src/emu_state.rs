use derive_new::new;
use std::time::Duration;

use super_yane::Console;

#[derive(new)]
pub struct EmuState {
    pub emu: Option<Console>,
    pub total_cycles: u64,
}
