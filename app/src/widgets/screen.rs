use iced::{
    Rectangle, Renderer, Theme,
    mouse::Cursor,
    widget::{
        canvas::{self, Image},
        image::{FilterMethod, Handle},
    },
};
use super_yane::Console;

use crate::application::Message;

pub struct Screen<'a> {
    pub frame_data: &'a [u8],
}

impl<Message> canvas::Program<Message> for Screen<'_> {
    type State = ();
    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: iced::Rectangle,
        cursor: Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let image = Image::new(Handle::from_rgba(256, 240, self.frame_data.to_vec()))
            .filter_method(FilterMethod::Nearest);
        frame.draw_image(
            Rectangle {
                x: 0.0,
                y: 0.0,
                ..bounds
            },
            image,
        );
        vec![frame.into_geometry()]
    }
}
