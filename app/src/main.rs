use log::*;
use simplelog::{ConfigBuilder, SimpleLogger};

mod application;
use application::Application;

fn main() {
    SimpleLogger::init(
        log::LevelFilter::Info,
        ConfigBuilder::new()
            .set_location_level(LevelFilter::Info)
            .add_filter_ignore_str("iced")
            .build(),
    )
    .unwrap();
    info!("Logger initialized");

    iced::application("Super Y.A.N.E.", Application::update, Application::view)
        .subscription(Application::subscription)
        .theme(Application::theme)
        .run()
        .unwrap();
}
