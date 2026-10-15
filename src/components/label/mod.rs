use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Text, TextStyleBuilder},
    mono_font::MonoTextStyle,
};
use embedded_graphics::geometry::Size;

use crate::common::{copy_truncated, DrawableComponent, State};
use heapless::String;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

const MAX_LABEL_LENGTH: usize = 32;

/// Represents a label component that can be updated and drawn.
pub struct Label {
    pub position: Point,
    pub size: Size,
    pub label: String<MAX_LABEL_LENGTH>,
    on_update: Option<fn(&mut Self, &State)>, // Function pointer
    pub style: MonoTextStyle<'static, Rgb565>,
    pub needs_redraw: bool,
    /// Colour behind the text. When set, a redraw erases the previous text;
    /// when `None` the text is drawn over whatever is below, so only use it
    /// for text that never changes.
    background: Option<Rgb565>,
    /// Area covered by the text at the last draw, to erase it on the next.
    drawn_area: Option<Rectangle>,
}

impl Label {
    /// Creates a new `Label` instance.
    pub fn new(position: Point, size: Size, label: &str, style: MonoTextStyle<'static, Rgb565>) -> Self {
        let mut label_str: String<MAX_LABEL_LENGTH> = String::new();
        copy_truncated(&mut label_str, label);
        Self {
            position,
            size,
            label: label_str,
            on_update: None, // No callback set initially
            style,
            needs_redraw: false,
            background: None,
            drawn_area: None,
        }
    }

    /// Sets the colour behind the text, so changing text erases the old one.
    pub fn with_background(mut self, color: Rgb565) -> Self {
        self.background = Some(color);
        self
    }

    /// Sets the `on_update` function pointer after creation.
    pub fn set_on_update(&mut self, callback: fn(&mut Self, &State)) {
        self.on_update = Some(callback);
    }

    /// Calls the `on_update` function if it exists.
    pub fn update(&mut self, state: &State) {
        if let Some(callback) = self.on_update {
            callback(self, state);
        }
    }

    /// Updates the label's text, truncating it to fit.
    /// Marks the label for redraw only if the text changed.
    pub fn set_label_text(&mut self, new_text: &str) {
        let mut text: String<MAX_LABEL_LENGTH> = String::new();
        copy_truncated(&mut text, new_text);
        if text != self.label {
            self.label = text;
            self.needs_redraw = true;
        }
    }

    /// Formats `args` into the label's text, truncating it to fit.
    pub fn set_label_fmt(&mut self, args: core::fmt::Arguments) {
        let mut buf = [0u8; MAX_LABEL_LENGTH];
        if let Ok(text) = format_no_std::show(&mut buf, args) {
            self.set_label_text(text);
        }
    }

    /// The text currently shown.
    pub fn text(&self) -> &str {
        &self.label
    }

    /// Draws the label on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let text_position =
            self.position + Point::new(self.size.width as i32 / 2, self.size.height as i32 / 2);

        let mut style = self.style;
        if self.background.is_some() {
            // The glyph cells overwrite the old text in one pass, without a
            // blank frame in between
            style.background_color = self.background;
        }
        let text = Text::with_text_style(
            &self.label,
            text_position,
            style,
            TextStyleBuilder::new()
                .alignment(embedded_graphics::text::Alignment::Center)
                .build(),
        );
        let new_area = text.bounding_box();
        text.draw(target)?;

        // Erase what the old text covered outside the new one
        if let (Some(color), Some(old_area)) = (self.background, self.drawn_area) {
            let fill = PrimitiveStyle::with_fill(color);
            for strip in uncovered(old_area, new_area) {
                strip.into_styled(fill).draw(target)?;
            }
        }
        self.drawn_area = Some(new_area);

        Ok(())
    }
}

/// The parts of `old` outside `new`, as up to four rectangles:
/// above, below, left and right of `new`.
fn uncovered(old: Rectangle, new: Rectangle) -> impl Iterator<Item = Rectangle> {
    let span = |r: &Rectangle| {
        let x0 = r.top_left.x;
        let y0 = r.top_left.y;
        (x0, y0, x0 + r.size.width as i32, y0 + r.size.height as i32)
    };
    let (ox0, oy0, ox1, oy1) = span(&old);
    let (nx0, ny0, nx1, ny1) = span(&new);
    // Rows of `old` that `new` also spans
    let my0 = ny0.clamp(oy0, oy1);
    let my1 = ny1.clamp(oy0, oy1);
    let rect = |x0: i32, y0: i32, x1: i32, y1: i32| {
        (x1 > x0 && y1 > y0)
            .then(|| Rectangle::new(Point::new(x0, y0), Size::new((x1 - x0) as u32, (y1 - y0) as u32)))
    };
    [
        rect(ox0, oy0, ox1, my0),
        rect(ox0, my1, ox1, oy1),
        rect(ox0, my0, nx0.clamp(ox0, ox1), my1),
        rect(nx1.clamp(ox0, ox1), my0, ox1, my1),
    ]
    .into_iter()
    .flatten()
}

impl DrawableComponent for Label {
    fn initialize(&mut self) {
        // Initialize any resources needed by the label
        self.needs_redraw = true; // Mark for initial draw
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        defmt::info!("Label draw");
        self.draw(target)
    }

    fn update(&mut self, state: &State) {
        self.update(state);
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        self.needs_redraw = needs_redraw;
    }

    fn cleanup(&mut self) {
        // Release any resources used by the label
        self.on_update = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;
    use profont::PROFONT_9_POINT;

    #[test]
    fn test_shorter_text_erases_old_pixels() {
        let style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);
        let mut label = Label::new(Point::new(0, 0), Size::new(60, 20), "10.0m", style)
            .with_background(Rgb565::BLACK);
        let mut display: MockDisplay<Rgb565> = MockDisplay::new();
        display.set_allow_overdraw(true);
        label.draw(&mut display).unwrap();

        label.set_label_text("9.9m");
        label.draw(&mut display).unwrap();

        // Same result as drawing the short text once on a clear screen
        let mut expected: MockDisplay<Rgb565> = MockDisplay::new();
        let mut fresh = Label::new(Point::new(0, 0), Size::new(60, 20), "9.9m", style)
            .with_background(Rgb565::BLACK);
        fresh.draw(&mut expected).unwrap();
        let lit = |display: &MockDisplay<Rgb565>| {
            display.affected_area().points().filter(|&p| display.get_pixel(p) == Some(Rgb565::WHITE)).count()
        };
        assert_eq!(lit(&display), lit(&expected));
    }

    #[test]
    fn test_same_text_does_not_need_redraw() {
        let style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);
        let mut label = Label::new(Point::new(0, 0), Size::new(60, 20), "1.0m", style);
        label.set_needs_redraw(false);
        label.set_label_text("1.0m");
        assert!(!label.needs_redraw);
        label.set_label_text("1.1m");
        assert!(label.needs_redraw);
    }
}
