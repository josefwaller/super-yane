use iced::{Font, Settings};
use log::*;
use simplelog::{ConfigBuilder, SimpleLogger};

mod application;
mod widgets;
use application::Application;

fn main() {
    SimpleLogger::init(
        log::LevelFilter::Debug,
        ConfigBuilder::new()
            .add_filter_allow_str("super_yane")
            .add_filter_allow_str("wdc65816")
            .build(),
    )
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
