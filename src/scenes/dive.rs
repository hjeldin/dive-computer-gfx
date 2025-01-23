use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor, Size, WebColors},
    primitives::PrimitiveStyleBuilder,
};
use profont::{PROFONT_18_POINT, PROFONT_9_POINT};

use crate::{
    common::State,
    components::{label::Label, plot::Plot},
    scene,
    utils::background::Background,
};
use core::format_args;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result::Ok;
use libm::ceil;

pub fn build_dive() -> scene::Scene {
    let mut scene = scene::Scene::new();
    let background = Background::new(Point::new(0, 16), Size::new(320, 240 - 16), None);

    let header_style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb565::CSS_DIM_GRAY)
        .build();

    let header = Background::new(Point::new(0, 50), Size::new(320, 16), Some(header_style));

    let title_style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);
    let value_style = MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::GREEN);

    let depth_lbl = Label::new(Point::new(27, 50), Size::new(36, 20), "Depth", title_style);

    let ndt_lbl = Label::new(Point::new(142, 50), Size::new(36, 20), "NDT", title_style);

    let runtime_lbl = Label::new(
        Point::new(252, 50),
        Size::new(36, 20),
        "Runtime",
        title_style,
    );

    let header_2 = Background::new(Point::new(0, 115), Size::new(320, 16), Some(header_style));

    let max_depth_lbl = Label::new(
        Point::new(27, 115),
        Size::new(65, 20),
        "Max Depth",
        title_style,
    );

    let ceiling_lbl = Label::new(
        Point::new(142, 115),
        Size::new(51, 20),
        "Ceiling",
        title_style,
    );

    let asc_rate_lbl = Label::new(
        Point::new(252, 115),
        Size::new(58, 20),
        "Asc Rate",
        title_style,
    );

    let mut depth_val = Label::new(Point::new(12, 82), Size::new(68, 24), "0.00", value_style);
    depth_val.set_on_update(|label: &mut Label, state: &State| {
        // Update the label text based on state
        let mut buf = [0u8; 32];
        let text = format_no_std::show(&mut buf, format_args!("{:.2}m", state.pressure));
        if let Ok(text) = text {
            label.set_label_text(&text);
        } else {
            // print error
            // println!("Error updating label text: {:?}", text);
        }
    });

    let mut ndt_val = Label::new(Point::new(125, 82), Size::new(60, 24), "0.00", value_style);
    ndt_val.set_on_update(|label: &mut Label, state: &State| {
        // Update the label text based on state
        let mut buf = [0u8; 32];
        let text = format_no_std::show(
            &mut buf,
            format_args!("{:3}min", ceil(state.pressure as f64) as u32),
        );
        if let Ok(text) = text {
            label.set_label_text(&text);
        } else {
            // print error
            // println!("Error updating label text: {:?}", text);
        }
    });

    let mut runtime_val = Label::new(Point::new(240, 82), Size::new(46, 24), "0.00", value_style);
    runtime_val.set_on_update(|label: &mut Label, state: &State| {
        // Update the label text based on state
        let mut buf = [0u8; 32];
        let text = format_no_std::show(
            &mut buf,
            format_args!("{:3}min", ceil(state.pressure as f64) as u32),
        );
        if let Ok(text) = text {
            label.set_label_text(&text);
        } else {
            // print error
            // println!("Error updating label text: {:?}", text);
        }
    });

    let mut max_depth_val = Label::new(Point::new(12, 146), Size::new(68, 24), "0.00", value_style);
    max_depth_val.set_on_update(|label: &mut Label, state: &State| {
        // Update the label text based on state
        let mut buf = [0u8; 32];
        let text = format_no_std::show(&mut buf, format_args!("{:.2}m", state.pressure));
        if let Ok(text) = text {
            label.set_label_text(&text);
        } else {
            // print error
            // println!("Error updating label text: {:?}", text);
        }
    });

    let mut ceiling_val = Label::new(Point::new(125, 146), Size::new(60, 24), "0.00", value_style);
    ceiling_val.set_on_update(|label: &mut Label, state: &State| {
        // Update the label text based on state
        let mut buf = [0u8; 32];
        let text = format_no_std::show(
            &mut buf,
            format_args!("{:3}min", ceil(state.pressure as f64) as u32),
        );
        if let Ok(text) = text {
            label.set_label_text(&text);
        } else {
            // print error
            // println!("Error updating label text: {:?}", text);
        }
    });

    let mut asc_rate_val = Label::new(Point::new(240, 146), Size::new(46, 24), "0.00", value_style);
    asc_rate_val.set_on_update(|label: &mut Label, state: &State| {
        // Update the label text based on state
        let mut buf = [0u8; 32];
        let text = format_no_std::show(
            &mut buf,
            format_args!("{:3}min", ceil(state.pressure as f64) as u32),
        );
        if let Ok(text) = text {
            label.set_label_text(&text);
        } else {
            // print error
            // println!("Error updating label text: {:?}", text);
        }
    });

    let mut depth_plot = Plot::new(
        Point::new(12, 180),
        Size::new(320 - 24, 40),
        MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE),
    );

    scene.add_component(crate::common::Drawable::Background(background));
    scene.add_component(crate::common::Drawable::Background(header));
    scene.add_component(crate::common::Drawable::Label(depth_lbl));
    scene.add_component(crate::common::Drawable::Label(ndt_lbl));
    scene.add_component(crate::common::Drawable::Label(runtime_lbl));

    scene.add_component(crate::common::Drawable::Label(depth_val));
    scene.add_component(crate::common::Drawable::Label(ndt_val));
    scene.add_component(crate::common::Drawable::Label(runtime_val));

    scene.add_component(crate::common::Drawable::Background(header_2));
    scene.add_component(crate::common::Drawable::Label(max_depth_lbl));
    scene.add_component(crate::common::Drawable::Label(ceiling_lbl));
    scene.add_component(crate::common::Drawable::Label(asc_rate_lbl));

    scene.add_component(crate::common::Drawable::Label(max_depth_val));
    scene.add_component(crate::common::Drawable::Label(ceiling_val));
    scene.add_component(crate::common::Drawable::Label(asc_rate_val));

    scene.add_component(crate::common::Drawable::Plot(depth_plot));

    scene.set_toggle_index(Some(0));

    scene
}
