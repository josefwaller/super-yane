use std::fs::File;

use iced::{Font, Settings};
use log::*;
use simplelog::{CombinedLogger, ConfigBuilder, SimpleLogger, WriteLogger};

mod application;
mod widgets;
use application::Application;

fn main() {
    let config = ConfigBuilder::new()
        .add_filter_allow_str("super_yane")
        .add_filter_allow_str("spc700")
        .add_filter_allow_str("wdc65816")
        .build();
    CombinedLogger::init(vec![
        WriteLogger::new(
            log::LevelFilter::Debug,
            config.clone(),
            File::create("./super_yane.log").unwrap(),
        ),
        SimpleLogger::new(log::LevelFilter::Debug, config),
    ])
    .unwrap();
    info!("Logger initialized");

    iced::application("Super Y.A.N.E.", Application::update, Application::view)
        .subscription(Application::subscription)
        .theme(Application::theme)
        .settings(Settings {
            id: None,
            fonts: vec![],
            default_font: Font::MONOSPACE,
            default_text_size: 12.into(),
            antialiasing: false,
        })
        .run()
        .unwrap();
}
