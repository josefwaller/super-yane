#[derive(Debug, Clone)]
pub enum Label {
    Reset,
    IrqNative,
    IrqEmu,
    NmiNative,
    NmiEmu,
    Location(String),
}

impl ToString for Label {
    fn to_string(&self) -> String {
        match self {
            Label::Reset => "reset".to_string(),
            Label::IrqNative => "irq_n".to_string(),
            Label::IrqEmu => "irq_e".to_string(),
            Label::NmiNative => "nmi_n".to_string(),
            Label::NmiEmu => "nmi_e".to_string(),
            Label::Location(s) => s.clone(),
        }
    }
}
