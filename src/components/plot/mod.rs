use embedded_plots::{
    axis::Scale,
    curve::{Curve, PlotPoint},
    single_plot::SinglePlot,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
    prelude::*,
    text::{Text, TextStyleBuilder},
    mono_font::MonoTextStyle,
};
use embedded_graphics::geometry::Size;

use crate::common::{DrawableComponent, State};
use heapless::Vec;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option::Some;
use core::option::Option::None;
use core::option::Option;

const MAX_PLOT_POINTS: usize = 4;

/// Represents a label component that can be updated and drawn.
pub struct Plot {
    pub position: Point,
    pub size: Size,
    on_update: Option<fn(&mut Self, &State)>, // Function pointer
    pub style: MonoTextStyle<'static, Rgb565>,
    pub data: Vec<PlotPoint, MAX_PLOT_POINTS>,
    pub needs_redraw: bool,
}

impl Plot {
    /// Creates a new `Label` instance.
    pub fn new(position: Point, size: Size, style: MonoTextStyle<'static, Rgb565>) -> Self {
        let mut data = Vec::new();
        let _ = data.push(PlotPoint{x: 0, y: 0 });
        let _ = data.push(PlotPoint{x: 1, y: 2 });
        let _ = data.push(PlotPoint{x: 2, y: 2 });
        let _ = data.push(PlotPoint{x: 3, y: 0 });
        Self {
            position,
            size,
            on_update: None, // No callback set initially
            style,
            data,
            needs_redraw: true,
        }
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

    /// Draws the label on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(
        &mut self,
        target: &mut D,
    ) -> Result<(), D::Error> {
        defmt::info!("Plot draw");
        let curve = Curve::from_data(self.data.as_slice());
        let curve_list = [(curve, Rgb565::YELLOW)];
        let plot = SinglePlot::new(&curve_list, Scale::RangeFraction(3), Scale::RangeFraction(2))
            .into_drawable(self.position,  self.position + self.size)
            .set_color(RgbColor::WHITE)
            .set_text_color(RgbColor::WHITE);

        plot.draw(target)?;
        self.needs_redraw = false;

        Ok(())
    }
}

impl DrawableComponent for Plot {
    fn initialize(&mut self) {
        // Initialize any resources needed by the plot
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
        // Release any resources used by the plot
        self.on_update = None;
    }
}
