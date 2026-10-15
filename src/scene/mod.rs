use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
};
use crate::common::{Action, Drawable, DrawableComponent, State, Toggleable};
use heapless::Vec;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

pub mod scene_manager;
const MAX_COMPONENTS: usize = 32;

pub struct Scene {
    components: Vec<Drawable, MAX_COMPONENTS>,
    current_toggle_index: Option<usize>
}

impl Scene {
    /// Create a new, empty scene.
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            current_toggle_index: None,
        }
    }

    /// Add a component to the scene.
    /// Initializes the component as part of adding it.
    pub fn add_component(&mut self, mut component: Drawable) {
        component.initialize();
        if self.components.push(component).is_err() {
            defmt::warn!("Scene is full, component dropped");
        }
    }

    /// Draw the components that changed since they were last drawn.
    /// Components are drawn in order, so a component on top of another is
    /// only correct if both are dirty together, as after `activate`.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        for component in &mut self.components {
            if component.needs_redraw() {
                component.draw(target)?;
                component.set_needs_redraw(false);
            }
        }
        Ok(())
    }

    /// Prepare the scene to be shown again after it was inactive.
    /// Components are kept, so switching back to a scene restores it.
    pub fn activate(&mut self) {
        for component in &mut self.components {
            component.set_needs_redraw(true);
        }
    }

    /// Clean up all components and clear the scene.
    /// Call this only when the scene is discarded: the scene is empty afterwards.
    pub fn cleanup(&mut self) {
        for component in &mut self.components {
            component.cleanup();
        }
        self.components.clear();
        self.current_toggle_index = None;
    }

    /// Select the component at `index`, or clear the selection with `None`.
    /// An index that does not point at a selectable component clears the selection.
    pub fn set_toggle_index(&mut self, index: Option<usize>) {
        self.select(index.filter(|&i| self.components.get(i).is_some_and(Drawable::is_selectable)));
    }

    /// Select the next selectable component, wrapping around.
    /// With no selection, selects the first selectable component.
    pub fn select_next_toggleable(&mut self) {
        let len = self.components.len();
        let start = self.current_toggle_index.map_or(0, |current| current + 1);
        let next = (0..len)
            .map(|offset| (start + offset) % len)
            .find(|&index| self.components[index].is_selectable());
        self.select(next);
    }

    /// Select the previous selectable component, wrapping around.
    /// With no selection, selects the last selectable component.
    pub fn select_prev_toggleable(&mut self) {
        let len = self.components.len();
        let start = self.current_toggle_index.unwrap_or(0) + len;
        let prev = (1..=len)
            .map(|offset| (start - offset) % len)
            .find(|&index| self.components[index].is_selectable());
        self.select(prev);
    }

    fn select(&mut self, index: Option<usize>) {
        if let Some(current) = self.current_toggle_index {
            self.components[current].toggle(false);
        }
        if let Some(next) = index {
            self.components[next].toggle(true);
        }
        self.current_toggle_index = index;
    }

    /// Click the selected component and return the action it requests.
    pub fn click_currently_toggled(&mut self) -> Action {
        match self.current_toggle_index {
            Some(index) => self.components[index].click(),
            None => Action::None,
        }
    }

    pub fn update(&mut self, state: &State) {
        for component in &mut self.components {
            component.update(state);
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::mock_display::MockDisplay;
    use embedded_graphics::geometry::Point;
    use embedded_graphics::mono_font::MonoTextStyle;
    use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
    use embedded_graphics::prelude::Size;
    use profont::PROFONT_9_POINT;
    use crate::components::button::Button;
    use crate::components::label::Label;

    fn button(label: &str) -> Drawable {
        Drawable::Button(Button::new(Point::new(0, 0), Size::new(10, 10), label))
    }

    fn label() -> Drawable {
        let style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);
        Drawable::Label(Label::new(Point::new(0, 0), Size::new(10, 10), "", style))
    }

    /// A scene laid out as [label, button, label, button].
    fn mixed_scene() -> Scene {
        let mut scene = Scene::new();
        scene.add_component(label());
        scene.add_component(button("Button 1"));
        scene.add_component(label());
        scene.add_component(button("Button 2"));
        scene
    }

    fn is_toggled(scene: &Scene, index: usize) -> bool {
        scene.components[index].is_toggled()
    }

    #[test]
    fn test_scene_new() {
        let scene = Scene::new();
        assert_eq!(scene.components.len(), 0);
        assert_eq!(scene.current_toggle_index, None);
    }

    #[test]
    fn test_add_component() {
        let mut scene = Scene::new();
        scene.add_component(button("Test Button"));
        assert_eq!(scene.components.len(), 1);
    }

    #[test]
    fn test_draw() {
        let mut scene = Scene::new();
        scene.add_component(button("Test Button"));

        let mut display = MockDisplay::new();
        display.set_allow_out_of_bounds_drawing(true);
        display.set_allow_overdraw(true);
        assert!(scene.draw(&mut display).is_ok());
        assert_ne!(display, MockDisplay::new());
    }

    #[test]
    fn test_cleanup() {
        let mut scene = Scene::new();
        scene.add_component(button("Test Button"));
        scene.select_next_toggleable();

        scene.cleanup();
        assert_eq!(scene.components.len(), 0);
        assert_eq!(scene.current_toggle_index, None);
    }

    #[test]
    fn test_activate_keeps_components() {
        let mut scene = mixed_scene();
        scene.activate();
        assert_eq!(scene.components.len(), 4);
    }

    #[test]
    fn test_select_next_toggleable() {
        let mut scene = mixed_scene();
        assert_eq!(scene.current_toggle_index, None);

        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, Some(1));
        assert!(is_toggled(&scene, 1));

        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, Some(3));
        assert!(!is_toggled(&scene, 1));
        assert!(is_toggled(&scene, 3));

        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, Some(1));
    }

    #[test]
    fn test_select_prev_toggleable() {
        let mut scene = mixed_scene();

        // With no selection, the last selectable component is selected
        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, Some(3));

        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, Some(1));
        assert!(!is_toggled(&scene, 3));
        assert!(is_toggled(&scene, 1));

        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, Some(3));
    }

    #[test]
    fn test_select_without_selectable_components() {
        let mut scene = Scene::new();
        scene.add_component(label());

        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, None);
        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, None);
    }

    #[test]
    fn test_set_toggle_index_rejects_unselectable() {
        let mut scene = mixed_scene();

        scene.set_toggle_index(Some(3));
        assert_eq!(scene.current_toggle_index, Some(3));
        assert!(is_toggled(&scene, 3));

        scene.set_toggle_index(Some(0));
        assert_eq!(scene.current_toggle_index, None);
        assert!(!is_toggled(&scene, 3));

        scene.set_toggle_index(Some(99));
        assert_eq!(scene.current_toggle_index, None);
    }

    #[test]
    fn test_click_currently_toggled() {
        let mut scene = Scene::new();
        let mut button = Button::new(Point::new(0, 0), Size::new(10, 10), "Test Button");
        button.set_on_click(|_| Action::SwitchScene(2));
        scene.add_component(Drawable::Button(button));

        assert_eq!(scene.click_currently_toggled(), Action::None);

        scene.select_next_toggleable();
        assert_eq!(scene.click_currently_toggled(), Action::SwitchScene(2));
    }

    #[test]
    fn test_update() {
        let mut scene = Scene::new();
        let style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);
        let mut label = Label::new(Point::new(0, 0), Size::new(10, 10), "", style);
        label.set_on_update(|label, state| label.set_label_fmt(format_args!("{}", state.battery)));
        scene.add_component(Drawable::Label(label));

        let mut state = State::new();
        state.battery = 42;
        scene.update(&state);

        match &scene.components[0] {
            Drawable::Label(label) => assert_eq!(label.label.as_str(), "42"),
            _ => unreachable!(),
        }
    }

    /// Counts the pixels a draw call touches, ignoring their colour.
    struct PixelCounter(usize);

    impl embedded_graphics::geometry::OriginDimensions for PixelCounter {
        fn size(&self) -> Size {
            Size::new(320, 240)
        }
    }

    impl DrawTarget for PixelCounter {
        type Color = Rgb565;
        type Error = core::convert::Infallible;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
        {
            self.0 += pixels.into_iter().count();
            Ok(())
        }
    }

    fn depth_label() -> Drawable {
        let style = MonoTextStyle::new(&PROFONT_9_POINT, Rgb565::WHITE);
        let mut label = Label::new(Point::new(0, 0), Size::new(60, 20), "0.0m", style);
        label.set_on_update(|label, state| label.set_label_fmt(format_args!("{:.1}m", state.pressure)));
        Drawable::Label(label)
    }

    #[test]
    fn test_draw_skips_clean_components() {
        let mut scene = Scene::new();
        scene.add_component(depth_label());
        let mut target = PixelCounter(0);

        scene.draw(&mut target).unwrap();
        assert!(target.0 > 0, "first draw paints the new component");

        target.0 = 0;
        scene.draw(&mut target).unwrap();
        assert_eq!(target.0, 0, "nothing changed, nothing is drawn");
    }

    #[test]
    fn test_update_redraws_only_when_text_changes() {
        let mut scene = Scene::new();
        scene.add_component(depth_label());
        let mut state = State::new();
        state.pressure = 12.34;
        scene.update(&state);
        let mut target = PixelCounter(0);
        scene.draw(&mut target).unwrap();

        // 12.31 still shows as 12.3m
        state.pressure = 12.31;
        scene.update(&state);
        target.0 = 0;
        scene.draw(&mut target).unwrap();
        assert_eq!(target.0, 0);

        state.pressure = 12.46;
        scene.update(&state);
        scene.draw(&mut target).unwrap();
        assert!(target.0 > 0);
    }

    #[test]
    fn test_activate_redraws_everything() {
        let mut scene = Scene::new();
        scene.add_component(depth_label());
        let mut target = PixelCounter(0);
        scene.draw(&mut target).unwrap();

        scene.activate();
        target.0 = 0;
        scene.draw(&mut target).unwrap();
        assert!(target.0 > 0);
    }

    #[test]
    fn test_selection_change_redraws_button() {
        let mut scene = Scene::new();
        scene.add_component(button("Button 1"));
        let mut target = PixelCounter(0);
        scene.draw(&mut target).unwrap();

        scene.select_next_toggleable();
        target.0 = 0;
        scene.draw(&mut target).unwrap();
        assert!(target.0 > 0);
    }

    #[test]
    fn test_empty_scene_toggle_operations() {
        let mut scene = Scene::new();

        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, None);

        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, None);

        assert_eq!(scene.click_currently_toggled(), Action::None);
    }
}
