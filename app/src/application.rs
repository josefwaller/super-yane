use iced::{
    Alignment::Center,
    Element, Event, Length, Subscription, Theme, event,
    widget::{
        button, column, container,
        image::{Handle, Image},
        row, text,
    },
    window,
};

use log::*;
use super_yane::Console;

#[derive(Debug, Clone)]
pub enum Message {
    NewFrame(),
    OnEvent(Event),
}

pub struct Application {
    red: u8,
    console: Console,
}

impl Default for Application {
    fn default() -> Self {
        Application {
            red: 0,
            console: Console::with_cartridge(include_bytes!("../roms/cputest-basic.sfc")),
        }
    }
}

impl Application {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::OnEvent(e) => info!("{:?}", e),
            _ => {}
        }
        self.red = self.red.wrapping_add(1);
    }
    pub fn view(&self) -> Element<'_, Message> {
        let data = [[self.red, 0, 0, 255]; 256 * 240];
        column![
            row![
                container(button("Pause")).width(Length::Shrink),
                Image::new(Handle::from_rgba(256, 240, data.as_flattened().to_vec()))
                    .height(Length::Fill)
                    .width(Length::FillPortion(50))
                    .content_fit(iced::ContentFit::Contain),
                container(text("Previous instructions")).width(Length::Shrink)
            ]
            .padding(10)
            .spacing(10)
            .height(Length::Fill),
            row![text("Thing One"), text("Thing Two"), text("thing three")]
                .align_y(Center)
                .height(Length::Fixed(200.0))
        ]
        .spacing(2)
        .align_x(Center)
        .width(Length::Fill)
        .into()
    }
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::frames().map(|_| Message::NewFrame()),
            event::listen().map(Message::OnEvent),
        ])
    }
    pub fn theme(&self) -> Theme {
        Theme::KanagawaDragon
    }
}
