use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, Rectangle, PrimitiveStyle, PrimitiveStyleBuilder, Triangle},
    text::{Text, TextStyleBuilder},
    mono_font::MonoTextStyle,
};

use crate::common::{DrawableComponent, State, Toggleable};
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option::Some;
use core::option::Option::None;
use core::option::Option;
use libm::{sin, cos, fabs};

/// Represents a compass component that can be updated and drawn.
pub struct Compass {
    pub position: Point,
    pub size: Size,
    on_update: Option<fn(&mut Self, &State)>, // Function pointer
    pub style: MonoTextStyle<'static, Rgb565>,
    pub needs_redraw: bool,
    pub is_toggled: bool,
    pub heading: f32,      // Current heading in degrees (0-359)
    pub set_course: f32,   // Set course in degrees (0-359)
}

impl Compass {
    /// Creates a new `Compass` instance.
    pub fn new(position: Point, size: Size, style: MonoTextStyle<'static, Rgb565>) -> Self {
        Self {
            position,
            size,
            on_update: None, // No callback set initially
            style,
            needs_redraw: true,
            is_toggled: false,
            heading: 0.0,
            set_course: 0.0,
        }
    }

    /// Sets the `on_update` function pointer after creation.
    pub fn set_on_update(&mut self, callback: fn(&mut Self, &State)) {
        self.on_update = Some(callback);
    }

    /// Calls the `on_update` function if it exists.
    pub fn update(&mut self, state: &State) {
        // Update heading and set_course from state
        self.heading = state.heading;
        self.set_course = state.set_course;
        self.needs_redraw = true;

        // Call the on_update callback if it exists
        if let Some(callback) = self.on_update {
            callback(self, state);
        }
    }

    /// Draws the compass on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        // Calculate center of the compass
        let center_x = self.position.x + (self.size.width as i32) / 2;
        let center_y = self.position.y + (self.size.height as i32) / 2;
        let center = Point::new(center_x, center_y);

        // Draw outer rectangle
        let rect_style = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb565::WHITE)
            .stroke_width(1)
            .fill_color(Rgb565::CSS_DARK_BLUE)
            .build();

        // Calculate top-left point for the rectangle
        let top_left = Point::new(center_x - self.size.width as i32 / 2, center_y - self.size.height as i32 / 2);
        let rect_size = Size::new(self.size.width, self.size.height);

        Rectangle::new(top_left, rect_size)
            .into_styled(rect_style)
            .draw(target)?;

        // Draw fixed heading indicator (red triangle pointing up)
        let heading_style = PrimitiveStyle::with_fill(Rgb565::RED);
        Triangle::new(
            Point::new(center_x, center_y - self.size.height as i32 / 2 + 5), // Tip at the top
            Point::new(center_x - 5, center_y - self.size.height as i32 / 2 + 15), // Left base
            Point::new(center_x + 5, center_y - self.size.height as i32 / 2 + 15), // Right base
        )
        .into_styled(heading_style)
        .draw(target)?;

        // Get the current heading and set course
        let heading = self.get_state_heading();
        let set_course = self.get_state_set_course();

        // Draw cardinal points (N, E, S, W) that rotate based on heading
        let text_style = self.style;

        // Calculate the offset angle for cardinal points based on heading
        // When heading is 0, North should be at the top
        // When heading is 90, East should be at the top, etc.

        // For a rectangle, we need to adjust the distance based on direction
        // Use width for east/west and height for north/south
        let width_factor = self.size.width as f32 * 0.4; // 80% of half-width
        let height_factor = self.size.height as f32 * 0.4; // 80% of half-height

        // North (offset by heading)
        let north_rad = (90.0 - (0.0 - heading)) * core::f32::consts::PI / 180.0;
        // Use cosine for x-component (affected by width) and sine for y-component (affected by height)
        let north_x = center_x + (width_factor * cos(north_rad as f64) as f32) as i32;
        let north_y = center_y - (height_factor * sin(north_rad as f64) as f32) as i32;
        Text::new(
            "N",
            Point::new(north_x - 4, north_y + 4),
            text_style,
        )
        .draw(target)?;

        // East (offset by heading)
        let east_rad = (90.0 - (90.0 - heading)) * core::f32::consts::PI / 180.0;
        let east_x = center_x + (width_factor * cos(east_rad as f64) as f32) as i32;
        let east_y = center_y - (height_factor * sin(east_rad as f64) as f32) as i32;
        Text::new(
            "E",
            Point::new(east_x - 4, east_y + 4),
            text_style,
        )
        .draw(target)?;

        // South (offset by heading)
        let south_rad = (90.0 - (180.0 - heading)) * core::f32::consts::PI / 180.0;
        let south_x = center_x + (width_factor * cos(south_rad as f64) as f32) as i32;
        let south_y = center_y - (height_factor * sin(south_rad as f64) as f32) as i32;
        Text::new(
            "S",
            Point::new(south_x - 4, south_y + 4),
            text_style,
        )
        .draw(target)?;

        // West (offset by heading)
        let west_rad = (90.0 - (270.0 - heading)) * core::f32::consts::PI / 180.0;
        let west_x = center_x + (width_factor * cos(west_rad as f64) as f32) as i32;
        let west_y = center_y - (height_factor * sin(west_rad as f64) as f32) as i32;
        Text::new(
            "W",
            Point::new(west_x - 4, west_y + 4),
            text_style,
        )
        .draw(target)?;

        // Draw intermediate points (NE, SE, SW, NW)
        // For diagonal directions, use a combination of width and height factors
        // Northeast (offset by heading)
        let ne_rad = (90.0 - (45.0 - heading)) * core::f32::consts::PI / 180.0;
        let ne_x = center_x + (width_factor * cos(ne_rad as f64) as f32) as i32;
        let ne_y = center_y - (height_factor * sin(ne_rad as f64) as f32) as i32;
        Text::new(
            "NE",
            Point::new(ne_x - 6, ne_y + 4),
            text_style,
        )
        .draw(target)?;

        // Southeast (offset by heading)
        let se_rad = (90.0 - (135.0 - heading)) * core::f32::consts::PI / 180.0;
        let se_x = center_x + (width_factor * cos(se_rad as f64) as f32) as i32;
        let se_y = center_y - (height_factor * sin(se_rad as f64) as f32) as i32;
        Text::new(
            "SE",
            Point::new(se_x - 6, se_y + 4),
            text_style,
        )
        .draw(target)?;

        // Southwest (offset by heading)
        let sw_rad = (90.0 - (225.0 - heading)) * core::f32::consts::PI / 180.0;
        let sw_x = center_x + (width_factor * cos(sw_rad as f64) as f32) as i32;
        let sw_y = center_y - (height_factor * sin(sw_rad as f64) as f32) as i32;
        Text::new(
            "SW",
            Point::new(sw_x - 6, sw_y + 4),
            text_style,
        )
        .draw(target)?;

        // Northwest (offset by heading)
        let nw_rad = (90.0 - (315.0 - heading)) * core::f32::consts::PI / 180.0;
        let nw_x = center_x + (width_factor * cos(nw_rad as f64) as f32) as i32;
        let nw_y = center_y - (height_factor * sin(nw_rad as f64) as f32) as i32;
        Text::new(
            "NW",
            Point::new(nw_x - 6, nw_y + 4),
            text_style,
        )
        .draw(target)?;

        // Draw set course indicator (green line) that rotates based on set course relative to heading
        if self.get_state_set_course() > 0.0 {
            // Calculate the angle for the set course relative to the heading
            // When set_course equals heading, the line should point up (same as the fixed heading indicator)
            let relative_course = set_course - heading;
            let course_rad = (90.0 - relative_course) * core::f32::consts::PI / 180.0;

            // Use width and height factors for the set course indicator
            let course_width_factor = width_factor * 0.75; // Slightly shorter than cardinal points
            let course_height_factor = height_factor * 0.75;

            let course_x = center_x + (course_width_factor * cos(course_rad as f64) as f32) as i32;
            let course_y = center_y - (course_height_factor * sin(course_rad as f64) as f32) as i32;

            let course_style = PrimitiveStyleBuilder::new()
                .stroke_color(Rgb565::GREEN)
                .stroke_width(2)
                .build();

            embedded_graphics::primitives::Line::new(center, Point::new(course_x, course_y))
                .into_styled(course_style)
                .draw(target)?;
        }

        // Draw current heading text in the center of the compass
        let mut heading_text_buf = [0u8; 32];
        let heading_text = format_no_std::show(
            &mut heading_text_buf,
            format_args!("{:.0}°", self.get_state_heading()),
        );

        if let Ok(heading_text) = heading_text {
            Text::new(
                &heading_text,
                Point::new(center_x - 15, center_y + 5), // Centered in the compass
                text_style,
            )
            .draw(target)?;
        }

        // If toggled, show set course text below the compass
        if self.is_toggled && self.get_state_set_course() > 0.0 {
            let mut course_text_buf = [0u8; 32];
            let course_text = format_no_std::show(
                &mut course_text_buf,
                format_args!("Course: {:.0}°", self.get_state_set_course()),
            );

            if let Ok(course_text) = course_text {
                Text::new(
                    &course_text,
                    Point::new(center_x - 30, center_y + self.size.height as i32 / 2 + 10),
                    text_style,
                )
                .draw(target)?;
            }

            // Show difference between heading and course
            let diff = self.get_heading_difference();
            let mut diff_text_buf = [0u8; 32];
            let diff_text = format_no_std::show(
                &mut diff_text_buf,
                format_args!("Diff: {:.0}°", diff),
            );

            if let Ok(diff_text) = diff_text {
                Text::new(
                    &diff_text,
                    Point::new(center_x - 25, center_y + self.size.height as i32 / 2 + 25),
                    text_style,
                )
                .draw(target)?;
            }
        }

        self.needs_redraw = false;
        Ok(())
    }

    // Helper method to get heading from state
    fn get_state_heading(&self) -> f32 {
        self.heading
    }

    // Helper method to get set course from state
    fn get_state_set_course(&self) -> f32 {
        self.set_course
    }

    // Calculate the difference between heading and set course
    fn get_heading_difference(&self) -> f32 {
        let heading = self.get_state_heading();
        let course = self.get_state_set_course();

        if course == 0.0 {
            return 0.0;
        }

        let diff = (course - heading) % 360.0;
        if diff > 180.0 {
            return diff - 360.0;
        } else if diff < -180.0 {
            return diff + 360.0;
        }
        diff
    }
}

impl DrawableComponent for Compass {
    fn initialize(&mut self) {
        // Initialize any resources needed by the compass
        self.needs_redraw = true; // Mark for initial draw
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        self.draw(target)
    }

    fn update(&mut self, state: &State) {
        self.update(state);
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        self.needs_redraw = needs_redraw;
    }

    fn cleanup(&mut self) {
        // Release any resources used by the compass
        self.on_update = None;
    }
}

impl Toggleable for Compass {
    fn toggle(&mut self, state: bool) {
        self.is_toggled = state;
        self.needs_redraw = true;
    }

    fn is_toggled(&self) -> bool {
        self.is_toggled
    }
}
