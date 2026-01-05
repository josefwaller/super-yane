use derive_new::new;
use iced::{
    Color, Point, Rectangle, Renderer, Size, Theme, color,
    mouse::Cursor,
    widget::{
        canvas::{self, Image, Stroke},
        image::{FilterMethod, Handle},
    },
};
use super_yane::Console;

/// Renders some abstract RGBA data to a canvas
#[derive(new)]
pub struct RgbaScreen<'a> {
    pub rgba_data: &'a [u8],
    pub width: u32,
    pub height: u32,
}

impl<'a> RgbaScreen<'a> {
    fn render_to_frame(&self, frame: &mut canvas::Frame, bounds: iced::Rectangle) {
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
    }
}

impl<Message> canvas::Program<Message> for RgbaScreen<'_> {
    type State = ();
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        self.render_to_frame(&mut frame, bounds);
        vec![frame.into_geometry()]
    }
}

/// Renders the console output and some debug information,
/// such as the location of every OAM sprite
#[derive(new)]
pub struct ConsoleDebugScreen<'a> {
    console: &'a Console,
    oam_outline_color: Option<Color>,
}

impl<Message> canvas::Program<Message> for ConsoleDebugScreen<'_> {
    type State = ();
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let pixel_size = (bounds.size().width / 256.0, bounds.size().height / 240.0);
        match self.oam_outline_color {
            None => {}
            Some(color) => {
                self.console.ppu().oam_sprites.iter().for_each(|sprite| {
                    let size = self.console.ppu().oam_sizes[sprite.size_select];
                    frame.stroke_rectangle(
                        Point::new(
                            sprite.x as f32 * pixel_size.0,
                            (sprite.y + 1) as f32 * pixel_size.1,
                        ),
                        Size::new(size.0 as f32 * pixel_size.0, size.1 as f32 * pixel_size.1),
                        Stroke::default().with_color(color),
                    );
                });
            }
        }
        vec![frame.into_geometry()]
    }
}
