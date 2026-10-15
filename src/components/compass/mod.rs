use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, Rectangle, PrimitiveStyle, Triangle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
    mono_font::MonoTextStyle,
};
use profont::{PROFONT_14_POINT, PROFONT_24_POINT};

use crate::common::{DrawableComponent, State, Toggleable};
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option::Some;
use core::option::Option::None;
use core::option::Option;
use libm::{fabsf, roundf, sqrtf};

/// Half of the heading range shown on the arc, in degrees.
const HALF_FIELD_OF_VIEW: f32 = 60.0;
/// How far the arc's ends drop below its centre, in pixels.
const ARC_DROP: f32 = 16.0;
/// Distance from the component's top edge to the top of the arc, in pixels.
const ARC_TOP: f32 = 16.0;
/// Gap between the arc's ends and the component's side edges, in pixels.
const ARC_SIDE_MARGIN: f32 = 8.0;
const COURSE_COLOR: Rgb565 = Rgb565::GREEN;

/// Represents a compass tape: a shallow arc that scrolls under a fixed pointer,
/// with the heading in large digits below and the turn to the set course under that.
pub struct Compass {
    pub position: Point,
    pub size: Size,
    on_update: Option<fn(&mut Self, &State)>, // Function pointer
    /// Style for the cardinal letters on the arc.
    pub style: MonoTextStyle<'static, Rgb565>,
    pub needs_redraw: bool,
    pub is_toggled: bool,
    pub heading: f32,      // Current heading in degrees (0-359)
    pub set_course: Option<f32>,   // Set course in degrees (0-359), None when unset
}

impl Compass {
    /// Creates a new `Compass` instance occupying the rectangle at `position` with `size`.
    pub fn new(position: Point, size: Size, style: MonoTextStyle<'static, Rgb565>) -> Self {
        Self {
            position,
            size,
            on_update: None, // No callback set initially
            style,
            needs_redraw: true,
            is_toggled: false,
            heading: 0.0,
            set_course: None,
        }
    }

    /// Sets the `on_update` function pointer after creation.
    pub fn set_on_update(&mut self, callback: fn(&mut Self, &State)) {
        self.on_update = Some(callback);
    }

    /// Copies heading and set course from the state, then calls `on_update` if set.
    pub fn update(&mut self, state: &State) {
        // The compass shows whole degrees, so smaller changes are invisible
        let shown = |course: Option<f32>| course.map(whole_degrees);
        if whole_degrees(self.heading) != whole_degrees(state.heading)
            || shown(self.set_course) != shown(state.set_course)
        {
            self.needs_redraw = true;
        }
        self.heading = state.heading;
        self.set_course = state.set_course;

        if let Some(callback) = self.on_update {
            callback(self, state);
        }
    }

    /// Draws the compass on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let bounds = Rectangle::new(self.position, self.size);
        bounds
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(target)?;

        let arc = ArcGeometry::new(bounds);
        self.draw_arc(&arc, target)?;
        self.draw_course(&arc, target)?;

        // Fixed pointer under the centre of the arc
        let tip = arc.point(0.0);
        Triangle::new(tip + Point::new(0, 2), tip + Point::new(-4, 8), tip + Point::new(4, 8))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
            .draw(target)?;

        self.draw_readout(&arc, bounds, target)?;

        if self.is_toggled {
            bounds
                .into_styled(PrimitiveStyle::with_stroke(Rgb565::YELLOW, 1))
                .draw(target)?;
        }

        self.needs_redraw = false;
        Ok(())
    }

    /// Draws the arc line, its tick marks and the cardinal letters above it.
    fn draw_arc<D: DrawTarget<Color = Rgb565>>(
        &self,
        arc: &ArcGeometry,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let line_style = PrimitiveStyle::with_stroke(Rgb565::WHITE, 1);
        arc.draw_span(-HALF_FIELD_OF_VIEW, HALF_FIELD_OF_VIEW, line_style, target)?;

        let letter_style = TextStyleBuilder::new()
            .alignment(Alignment::Center)
            .baseline(Baseline::Middle)
            .build();

        for bearing in (0..360).step_by(5) {
            let offset = wrap_degrees(bearing as f32 - self.heading);
            if fabsf(offset) > HALF_FIELD_OF_VIEW {
                continue;
            }
            let letter = match bearing {
                0 => Some("N"),
                90 => Some("E"),
                180 => Some("S"),
                270 => Some("W"),
                _ => None,
            };
            match letter {
                Some(letter) => {
                    Text::with_text_style(letter, arc.outward(offset, 8.0), self.style, letter_style)
                        .draw(target)?;
                }
                None => {
                    let length = if bearing % 45 == 0 { 7.0 } else if bearing % 15 == 0 { 5.0 } else { 2.0 };
                    Line::new(arc.point(offset), arc.outward(offset, length))
                        .into_styled(line_style)
                        .draw(target)?;
                }
            }
        }
        Ok(())
    }

    /// Draws the set course as a green span from the pointer towards the course,
    /// with a marker where the course lies on the arc.
    fn draw_course<D: DrawTarget<Color = Rgb565>>(
        &self,
        arc: &ArcGeometry,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let Some(offset) = self.course_offset() else {
            return Ok(());
        };
        let end = offset.clamp(-HALF_FIELD_OF_VIEW, HALF_FIELD_OF_VIEW);
        arc.draw_span(0.0, end, PrimitiveStyle::with_stroke(COURSE_COLOR, 3), target)?;
        if fabsf(offset) <= HALF_FIELD_OF_VIEW {
            Line::new(arc.point(end), arc.outward(end, 8.0))
                .into_styled(PrimitiveStyle::with_stroke(COURSE_COLOR, 2))
                .draw(target)?;
        }
        Ok(())
    }

    /// Draws the heading in large digits and, when a course is set, the turn
    /// needed to reach it below: an arrow for the direction and the angle in degrees.
    /// The block is centred in the space between the pointer and the bottom edge.
    fn draw_readout<D: DrawTarget<Color = Rgb565>>(
        &self,
        arc: &ArcGeometry,
        bounds: Rectangle,
        target: &mut D,
    ) -> Result<(), D::Error> {
        const GAP: i32 = 4;
        const ARROW_WIDTH: i32 = 6;
        let heading_font = &PROFONT_24_POINT;
        let turn_font = &PROFONT_14_POINT;
        let turn = self.course_offset().map(|offset| roundf(offset) as i32);

        let block_height = heading_font.character_size.height as i32
            + turn.map_or(0, |_| GAP + turn_font.character_size.height as i32);
        let space_top = arc.point(0.0).y + 10;
        let space_bottom = bounds.top_left.y + bounds.size.height as i32;
        let top = space_top + ((space_bottom - space_top - block_height) / 2).max(0);
        let center_x = arc.point(0.0).x;
        let top_left = TextStyleBuilder::new().alignment(Alignment::Left).baseline(Baseline::Top).build();

        let mut heading_buf = [0u8; 8];
        if let Ok(heading_text) = format_no_std::show(
            &mut heading_buf,
            format_args!("{}\u{b0}", whole_degrees(self.heading)),
        ) {
            Text::with_text_style(
                heading_text,
                Point::new(center_x, top),
                MonoTextStyle::new(heading_font, Rgb565::WHITE),
                TextStyleBuilder::new().alignment(Alignment::Center).baseline(Baseline::Top).build(),
            )
            .draw(target)?;
        }

        let Some(turn) = turn else {
            return Ok(());
        };
        let mut turn_buf = [0u8; 8];
        let Ok(turn_text) = format_no_std::show(&mut turn_buf, format_args!("{}\u{b0}", turn.abs())) else {
            return Ok(());
        };
        let turn_top = top + heading_font.character_size.height as i32 + GAP;
        let arrow_space = if turn == 0 { 0 } else { ARROW_WIDTH + 3 };
        let text_width = turn_text.chars().count() as i32 * turn_font.character_size.width as i32;
        let mut x = center_x - (arrow_space + text_width) / 2;

        if turn != 0 {
            // Arrow pointing the way to turn
            let (back, tip) = if turn > 0 { (x, x + ARROW_WIDTH) } else { (x + ARROW_WIDTH, x) };
            let mid = turn_top + turn_font.character_size.height as i32 / 2;
            Triangle::new(Point::new(back, mid - 5), Point::new(back, mid + 5), Point::new(tip, mid))
                .into_styled(PrimitiveStyle::with_fill(COURSE_COLOR))
                .draw(target)?;
            x += arrow_space;
        }

        Text::with_text_style(
            turn_text,
            Point::new(x, turn_top),
            MonoTextStyle::new(turn_font, COURSE_COLOR),
            top_left,
        )
        .draw(target)?;
        Ok(())
    }

    /// Signed turn from the heading to the set course in [-180, 180],
    /// or `None` when no course is set.
    fn course_offset(&self) -> Option<f32> {
        self.set_course.map(|course| wrap_degrees(course - self.heading))
    }
}

/// Maps an offset from the heading onto a shallow circular arc.
/// Offsets are spread linearly across the arc's width; the circle only gives the curve.
struct ArcGeometry {
    center_x: f32,
    top_y: f32,
    half_width: f32,
    radius: f32,
}

impl ArcGeometry {
    fn new(bounds: Rectangle) -> Self {
        let half_width = bounds.size.width as f32 / 2.0 - ARC_SIDE_MARGIN;
        Self {
            center_x: bounds.top_left.x as f32 + bounds.size.width as f32 / 2.0,
            top_y: bounds.top_left.y as f32 + ARC_TOP,
            half_width,
            // Circle through the arc's top and both ends
            radius: (half_width * half_width + ARC_DROP * ARC_DROP) / (2.0 * ARC_DROP),
        }
    }

    fn position(&self, offset: f32) -> (f32, f32) {
        let dx = offset / HALF_FIELD_OF_VIEW * self.half_width;
        let y = self.top_y + self.radius - sqrtf(self.radius * self.radius - dx * dx);
        (self.center_x + dx, y)
    }

    /// Point on the arc at `offset` degrees from the heading.
    fn point(&self, offset: f32) -> Point {
        let (x, y) = self.position(offset);
        Point::new(roundf(x) as i32, roundf(y) as i32)
    }

    /// Point `distance` pixels outside the arc at `offset`, along the circle's radius.
    fn outward(&self, offset: f32, distance: f32) -> Point {
        let (x, y) = self.position(offset);
        let center_y = self.top_y + self.radius;
        let scale = distance / self.radius;
        Point::new(
            roundf(x + (x - self.center_x) * scale) as i32,
            roundf(y + (y - center_y) * scale) as i32,
        )
    }

    /// Draws the arc between two offsets as short line segments.
    fn draw_span<D: DrawTarget<Color = Rgb565>>(
        &self,
        from: f32,
        to: f32,
        style: PrimitiveStyle<Rgb565>,
        target: &mut D,
    ) -> Result<(), D::Error> {
        const STEPS_PER_DEGREE: f32 = 1.0;
        let steps = (fabsf(to - from) * STEPS_PER_DEGREE) as i32 + 1;
        let mut previous = self.point(from);
        for step in 1..=steps {
            let next = self.point(from + (to - from) * step as f32 / steps as f32);
            Line::new(previous, next).into_styled(style).draw(target)?;
            previous = next;
        }
        Ok(())
    }
}

/// Wraps an angle in degrees into the range [-180, 180].
fn wrap_degrees(degrees: f32) -> f32 {
    let wrapped = degrees % 360.0;
    if wrapped > 180.0 {
        wrapped - 360.0
    } else if wrapped < -180.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}

/// Heading rounded to whole degrees in 0..360, so 359.6 shows as 0.
fn whole_degrees(degrees: f32) -> i32 {
    (roundf(degrees) as i32).rem_euclid(360)
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

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
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
        if self.is_toggled != state {
            self.is_toggled = state;
            self.needs_redraw = true;
        }
    }

    fn is_toggled(&self) -> bool {
        self.is_toggled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;
    use profont::PROFONT_9_POINT;

    fn compass(heading: f32, set_course: Option<f32>) -> Compass {
        let mut compass = Compass::new(
            Point::new(0, 0),
            Size::new(64, 64),
            MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE),
        );
        compass.heading = heading;
        compass.set_course = set_course;
        compass
    }

    #[test]
    fn test_wrap_degrees() {
        assert_eq!(wrap_degrees(190.0), -170.0);
        assert_eq!(wrap_degrees(-190.0), 170.0);
        assert_eq!(wrap_degrees(45.0), 45.0);
    }

    #[test]
    fn test_whole_degrees_wraps_at_north() {
        assert_eq!(whole_degrees(359.6), 0);
        assert_eq!(whole_degrees(235.2), 235);
    }

    #[test]
    fn test_course_offset() {
        assert_eq!(compass(235.0, None).course_offset(), None);
        assert_eq!(compass(235.0, Some(265.0)).course_offset(), Some(30.0));
        assert_eq!(compass(350.0, Some(20.0)).course_offset(), Some(30.0));
        assert_eq!(compass(20.0, Some(350.0)).course_offset(), Some(-30.0));
        // North is a real course, not "unset"
        assert_eq!(compass(330.0, Some(0.0)).course_offset(), Some(30.0));
        assert_eq!(compass(0.0, Some(0.0)).course_offset(), Some(0.0));
    }

    #[test]
    fn test_arc_ends_drop_below_centre() {
        let arc = ArcGeometry::new(Rectangle::new(Point::new(0, 0), Size::new(320, 56)));
        let centre = arc.point(0.0);
        let left = arc.point(-HALF_FIELD_OF_VIEW);
        let right = arc.point(HALF_FIELD_OF_VIEW);
        assert_eq!(centre, Point::new(160, ARC_TOP as i32));
        assert_eq!(left.y, centre.y + ARC_DROP as i32);
        assert_eq!(right.y, left.y);
        assert_eq!(right.x - centre.x, centre.x - left.x);
    }

    #[test]
    fn test_draw_with_and_without_course() {
        for (heading, course) in [(0.0, None), (235.0, Some(265.0)), (10.0, Some(300.0)), (359.6, None), (0.0, Some(0.0))] {
            let mut display = MockDisplay::new();
            display.set_allow_out_of_bounds_drawing(true);
            display.set_allow_overdraw(true);
            assert!(compass(heading, course).draw(&mut display).is_ok());
        }
    }
}
