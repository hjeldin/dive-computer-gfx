use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
    mono_font::MonoTextStyle,
};
use embedded_graphics::geometry::Size;
use libm::{ceilf, roundf};

use crate::common::{DrawableComponent, State};
use heapless::Vec;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option::Some;

/// Number of depth samples kept. When full, every other sample is dropped and
/// the sample interval doubles, so the whole dive always fits.
const MAX_SAMPLES: usize = 64;
/// Sample interval at the start of a dive, in milliseconds.
const INITIAL_INTERVAL_MS: u32 = 5_000;
/// Shortest time span the time axis shows, in milliseconds.
const MIN_TIME_SPAN_MS: u32 = 60_000;
/// Shallowest depth the depth axis shows, in metres.
const MIN_DEPTH_SPAN_M: f32 = 10.0;
/// Room left of the plot area for depth labels, and below it for time labels.
const LEFT_MARGIN: i32 = 22;
const BOTTOM_MARGIN: i32 = 12;

const PROFILE_COLOR: Rgb565 = Rgb565::YELLOW;
const FILL_COLOR: Rgb565 = Rgb565::new(0, 8, 8);
const GRID_COLOR: Rgb565 = Rgb565::CSS_DIM_GRAY;

/// Dive profile: depth over dive time, with depth increasing downwards.
pub struct DepthProfile {
    pub position: Point,
    pub size: Size,
    /// Style for the axis labels.
    pub style: MonoTextStyle<'static, Rgb565>,
    pub needs_redraw: bool,
    /// Depth in metres sampled at `0, interval, 2 * interval, ...` of dive time.
    samples: Vec<f32, MAX_SAMPLES>,
    interval_ms: u32,
    /// Latest dive time and depth, drawn as the live end of the profile.
    now_ms: u32,
    depth_now: f32,
}

impl DepthProfile {
    /// Creates a new, empty `DepthProfile` occupying the rectangle at `position` with `size`.
    pub fn new(position: Point, size: Size, style: MonoTextStyle<'static, Rgb565>) -> Self {
        Self {
            position,
            size,
            style,
            needs_redraw: true,
            samples: Vec::new(),
            interval_ms: INITIAL_INTERVAL_MS,
            now_ms: 0,
            depth_now: 0.0,
        }
    }

    /// Records the depth at the current dive time.
    /// A dive time lower than the last one means a new dive, and clears the profile.
    /// Only a new sample, a new dive or a depth change of 0.1 m marks it for redraw.
    pub fn record(&mut self, dive_time_ms: u32, depth: f32) {
        if dive_time_ms < self.now_ms {
            self.samples.clear();
            self.interval_ms = INITIAL_INTERVAL_MS;
            self.needs_redraw = true;
        }
        if tenths(depth) != tenths(self.depth_now) {
            self.needs_redraw = true;
        }
        self.now_ms = dive_time_ms;
        self.depth_now = depth;

        while self.samples.len() as u32 * self.interval_ms <= dive_time_ms {
            if self.samples.is_full() {
                self.halve_resolution();
            }
            // Samples skipped between updates take the current depth
            let _ = self.samples.push(depth);
            self.needs_redraw = true;
        }
    }

    /// Keeps every other sample and doubles the interval between them.
    fn halve_resolution(&mut self) {
        let kept = (self.samples.len() + 1) / 2;
        for index in 0..kept {
            self.samples[index] = self.samples[index * 2];
        }
        self.samples.truncate(kept);
        self.interval_ms *= 2;
    }

    /// Profile points as (dive time ms, depth m): the samples, then the live point.
    fn points(&self) -> impl Iterator<Item = (u32, f32)> + '_ {
        let samples = self
            .samples
            .iter()
            .enumerate()
            .map(|(index, &depth)| (index as u32 * self.interval_ms, depth));
        let last_sample_ms = self.samples.len().checked_sub(1).map(|last| last as u32 * self.interval_ms);
        let live = last_sample_ms.is_some_and(|last| last < self.now_ms);
        samples.chain(live.then_some((self.now_ms, self.depth_now)))
    }

    /// Draws the depth profile on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let area = Rectangle::new(
            self.position + Point::new(LEFT_MARGIN, 0),
            self.size - Size::new(LEFT_MARGIN as u32, BOTTOM_MARGIN as u32),
        );
        let Some(bottom_right) = area.bottom_right() else {
            return Ok(());
        };

        // Clear the previous profile: the scales may have changed
        Rectangle::new(self.position, self.size)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(target)?;

        let max_depth = self.points().map(|(_, depth)| depth).fold(0.0, f32::max);
        let depth_span = round_up(max_depth, 5.0).max(MIN_DEPTH_SPAN_M);
        let time_span_ms = round_up(self.now_ms as f32, 60_000.0).max(MIN_TIME_SPAN_MS as f32);

        let to_point = |(time_ms, depth): (u32, f32)| {
            Point::new(
                area.top_left.x + scale(time_ms as f32 / time_span_ms, area.size.width),
                area.top_left.y + scale(depth / depth_span, area.size.height),
            )
        };

        self.draw_axes(area, bottom_right, depth_span, time_span_ms, target)?;

        if self.samples.is_empty() {
            self.needs_redraw = false;
            return Ok(());
        }

        // Water above the profile, then the profile line on top
        let fill = PrimitiveStyle::with_stroke(FILL_COLOR, 1);
        let mut previous = to_point((0, self.samples[0]));
        for point in self.points().map(to_point).skip(1) {
            for x in previous.x..point.x {
                let y = previous.y + (point.y - previous.y) * (x - previous.x) / (point.x - previous.x);
                Line::new(Point::new(x, area.top_left.y + 1), Point::new(x, y))
                    .into_styled(fill)
                    .draw(target)?;
            }
            previous = point;
        }

        let line = PrimitiveStyle::with_stroke(PROFILE_COLOR, 2);
        let mut previous = to_point((0, self.samples[0]));
        for point in self.points().map(to_point).skip(1) {
            Line::new(previous, point).into_styled(line).draw(target)?;
            previous = point;
        }

        self.needs_redraw = false;
        Ok(())
    }

    /// Draws depth grid lines with metre labels on the left, the surface line on top,
    /// and minute ticks with labels along the bottom.
    fn draw_axes<D: DrawTarget<Color = Rgb565>>(
        &self,
        area: Rectangle,
        bottom_right: Point,
        depth_span: f32,
        time_span_ms: f32,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let grid = PrimitiveStyle::with_stroke(GRID_COLOR, 1);
        let right_label = TextStyleBuilder::new().alignment(Alignment::Right).baseline(Baseline::Middle).build();
        let mut buf = [0u8; 8];

        let depth_step = tick_step(depth_span as u32, &[5, 10, 20, 50]);
        for depth in (0..=depth_span as u32).step_by(depth_step as usize) {
            let y = area.top_left.y + scale(depth as f32 / depth_span, area.size.height);
            Line::new(Point::new(area.top_left.x, y), Point::new(bottom_right.x, y))
                .into_styled(grid)
                .draw(target)?;
            // The bottom line carries the time labels instead
            if depth as f32 >= depth_span {
                continue;
            }
            if let Ok(text) = format_no_std::show(&mut buf, format_args!("{}m", depth)) {
                Text::with_text_style(text, Point::new(area.top_left.x - 3, y), self.style, right_label)
                    .draw(target)?;
            }
        }

        // Surface
        Line::new(area.top_left, Point::new(bottom_right.x, area.top_left.y))
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
            .draw(target)?;

        let minutes = (time_span_ms / 60_000.0) as u32;
        let minute_step = tick_step(minutes, &[1, 2, 5, 10, 15, 30, 60]);
        for minute in (0..=minutes).step_by(minute_step as usize) {
            let x = area.top_left.x + scale(minute as f32 / minutes as f32, area.size.width);
            Line::new(Point::new(x, bottom_right.y), Point::new(x, bottom_right.y + 2))
                .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
                .draw(target)?;
            // Keep the last label inside the component
            let alignment = if minute == minutes { Alignment::Right } else { Alignment::Center };
            let label_style = TextStyleBuilder::new().alignment(alignment).baseline(Baseline::Top).build();
            if let Ok(text) = format_no_std::show(&mut buf, format_args!("{}'", minute)) {
                Text::with_text_style(text, Point::new(x, bottom_right.y + 3), self.style, label_style)
                    .draw(target)?;
            }
        }
        Ok(())
    }
}

/// Depth in whole tenths of a metre, the precision the dive computer shows.
fn tenths(depth: f32) -> i32 {
    roundf(depth * 10.0) as i32
}

/// Rounds `value` up to a multiple of `step`.
fn round_up(value: f32, step: f32) -> f32 {
    ceilf(value / step) * step
}

/// Pixel offset of `fraction` (0 to 1) along `length` pixels.
fn scale(fraction: f32, length: u32) -> i32 {
    (fraction * (length - 1) as f32) as i32
}

/// Smallest step from `steps` that splits `span` into at most four parts.
fn tick_step(span: u32, steps: &[u32]) -> u32 {
    steps
        .iter()
        .copied()
        .find(|&step| span / step <= 4)
        .unwrap_or(span.max(1))
}

impl DrawableComponent for DepthProfile {
    fn initialize(&mut self) {
        self.needs_redraw = true;
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        self.draw(target)
    }

    fn update(&mut self, state: &State) {
        self.record(state.dive_time_ms, state.depth_m);
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        self.needs_redraw = needs_redraw;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;
    use profont::PROFONT_7_POINT;

    fn profile() -> DepthProfile {
        DepthProfile::new(
            Point::new(0, 0),
            Size::new(64, 64),
            MonoTextStyle::new(&PROFONT_7_POINT, Rgb565::WHITE),
        )
    }

    #[test]
    fn test_samples_at_fixed_interval() {
        let mut profile = profile();
        profile.record(0, 1.0);
        profile.record(4_000, 2.0);
        profile.record(5_000, 3.0);
        profile.record(12_000, 4.0);
        // 12 s covers samples at 0, 5 and 10 s
        assert_eq!(profile.samples.as_slice(), &[1.0, 3.0, 4.0]);
    }

    #[test]
    fn test_full_buffer_halves_resolution() {
        let mut profile = profile();
        for index in 0..MAX_SAMPLES as u32 {
            profile.record(index * INITIAL_INTERVAL_MS, index as f32);
        }
        assert!(profile.samples.is_full());

        profile.record(MAX_SAMPLES as u32 * INITIAL_INTERVAL_MS, 99.0);
        assert_eq!(profile.interval_ms, 2 * INITIAL_INTERVAL_MS);
        assert_eq!(profile.samples.len(), MAX_SAMPLES / 2 + 1);
        assert_eq!(profile.samples[1], 2.0);
        assert_eq!(*profile.samples.last().unwrap(), 99.0);
    }

    #[test]
    fn test_points_end_at_live_depth() {
        let mut profile = profile();
        profile.record(0, 0.0);
        profile.record(7_000, 6.0);
        let mut points = profile.points();
        assert_eq!(points.next(), Some((0, 0.0)));
        assert_eq!(points.next(), Some((5_000, 6.0)));
        assert_eq!(points.next(), Some((7_000, 6.0)));
        assert_eq!(points.next(), None);
    }

    #[test]
    fn test_new_dive_clears_profile() {
        let mut profile = profile();
        profile.record(0, 0.0);
        profile.record(30_000, 10.0);
        profile.record(0, 1.0);
        assert_eq!(profile.samples.as_slice(), &[1.0]);
    }

    #[test]
    fn test_tick_step() {
        assert_eq!(tick_step(10, &[5, 10, 20]), 5);
        assert_eq!(tick_step(30, &[5, 10, 20]), 10);
        assert_eq!(tick_step(1, &[1, 2, 5]), 1);
    }

    #[test]
    fn test_draw_empty_flat_and_long_dives() {
        let mut empty = profile();
        let mut flat = profile();
        flat.record(0, 0.0);
        flat.record(20_000, 0.0);
        let mut long = profile();
        for second in (0..3_600).step_by(7) {
            long.record(second * 1_000, (second % 40) as f32);
        }
        for profile in [&mut empty, &mut flat, &mut long] {
            let mut display = MockDisplay::new();
            display.set_allow_out_of_bounds_drawing(true);
            display.set_allow_overdraw(true);
            assert!(profile.draw(&mut display).is_ok());
        }
    }
}
