pub enum Label {
    EntryPoint,
    Location(String),
}

impl ToString for Label {
    fn to_string(&self) -> String {
        match self {
            Label::EntryPoint => "entry_point".to_string(),
            Label::Location(s) => s.clone(),
        }
    }
}
