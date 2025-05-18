use embedded_graphics::geometry::Size;
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{CornerRadii, PrimitiveStyleBuilder, Rectangle, RoundedRectangle, StyledDrawable},
    text::{Text, TextStyleBuilder},
};
use profont::PROFONT_9_POINT;
use core::result::*;
use core::result::Result::Ok;
use core::format_args;

pub struct BatteryIndicator {
    level: u8,
}

impl BatteryIndicator {
    pub fn new(level: u8) -> Self {
        Self { level }
    }

    fn hex_to_percentage(&self, hex_value: u8) -> u8 {
        // Scale the value from 0-255 to 0-100
        ((hex_value as u32 * 100) / 255) as u8
    }
    
    pub fn set_level(&mut self, level: u8) {
        self.level = level;
    }

    pub fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        let percentage = self.hex_to_percentage(self.level);

        let rectangle = RoundedRectangle::new(
            Rectangle::new(Point::zero(), Size::new(36, 16)),
            CornerRadii::new(Size::new(2, 2)),
        );

        let style = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLACK)
            .stroke_color(Rgb565::WHITE)
            .stroke_width(1)
            .build();
        let _ = rectangle.draw_styled(&style, target);

        let cap = RoundedRectangle::new(
            Rectangle::new(Point::new(35, 4), Size::new(3, 8)),
            CornerRadii::new(Size::new(2, 2)),
        );
        let _ = cap.draw_styled(&style, target);
        
        if percentage == 0 {
            let inner_rect_style = PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::CSS_DIM_GRAY)
                .build();
            let rectangle = RoundedRectangle::new(
                Rectangle::new(Point::new(3, 2), Size::new(30, 12)),
                CornerRadii::new(Size::new(2, 2)),
            );
            let _ = rectangle.draw_styled(&inner_rect_style, target);
        } else {
            for index in 0..(percentage - 1) / 25 {
                let inner_rect_style = PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::CSS_DARK_GREEN)
                    .build();
                let rectangle = RoundedRectangle::new(
                    Rectangle::new(
                        Point::new(index as i32 * 10 + 3, 2),
                        Size::new(10, 12),
                    ),
                    CornerRadii::new(Size::new(2, 2)),
                );
                let _ = rectangle.draw_styled(&inner_rect_style, target);
            }
        }

        // Style objects
        let text_style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);

        let mut buf = [0u8; 5];
        let text = format_no_std::show(
            &mut buf,
            format_args!("{:02}%", percentage),
        )
        .unwrap_or("xx%");
        Text::with_text_style(
            &text,
            Point::new(18, 10),
            text_style,
            TextStyleBuilder::new()
                .alignment(embedded_graphics::text::Alignment::Center)
                .build(),
        )
        .draw(target)?;

        Ok(())
    }
}
