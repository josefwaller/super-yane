use derive_new::new;
use iced::{
    Rectangle, Renderer, Theme,
    mouse::Cursor,
    widget::{
        canvas::{self, Image},
        image::{FilterMethod, Handle},
    },
};

#[derive(new)]
pub struct Screen<'a> {
    pub rgba_data: &'a [u8],
    pub width: u32,
    pub height: u32,
}

impl<Message> canvas::Program<Message> for Screen<'_> {
    type State = ();
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        cursor: Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let image = Image::new(Handle::from_rgba(
            self.width,
            self.height,
            self.rgba_data.to_vec(),
        ))
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
