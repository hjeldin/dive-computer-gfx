use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::Rgb565,
};
use crate::common::{Action, Clickable, Drawable, DrawableComponent, State, Toggleable};
use crate::components::popup::Popup;
use heapless::Vec;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

pub mod scene_manager;
/// Room for the largest scene, settings, with a few spare. `test_scenes_fit` checks it.
pub(crate) const MAX_COMPONENTS: usize = 26;

pub struct Scene {
    components: Vec<Drawable, MAX_COMPONENTS>,
    current_toggle_index: Option<usize>,
    /// A popup shown over the components. While it is open it gets all the
    /// input, and the components below are not drawn.
    popup: Option<Popup>,
}

impl Scene {
    /// Create a new, empty scene.
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            current_toggle_index: None,
            popup: None,
        }
    }

    /// Shows `popup` over the scene, replacing any popup already shown.
    /// Selection and clicks go to the popup until one of its buttons is clicked.
    pub fn show_popup(&mut self, mut popup: Popup) {
        popup.initialize();
        self.popup = Some(popup);
    }

    /// Returns true while a popup is shown.
    pub fn has_popup(&self) -> bool {
        self.popup.is_some()
    }

    /// Add a component to the scene.
    /// Initializes the component as part of adding it.
    /// Number of components in the scene.
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

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
        if let Some(popup) = &mut self.popup {
            if popup.needs_redraw() {
                popup.draw(target)?;
                popup.set_needs_redraw(false);
            }
            return Ok(());
        }
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
        self.popup = None;
    }

    /// Select the component at `index`, or clear the selection with `None`.
    /// An index that does not point at a selectable component clears the selection.
    pub fn set_toggle_index(&mut self, index: Option<usize>) {
        self.select(index.filter(|&i| self.components.get(i).is_some_and(Drawable::is_selectable)));
    }

    /// The selected component, if it is editing its value.
    fn editing_index(&self) -> Option<usize> {
        self.current_toggle_index.filter(|&index| self.components[index].is_editing())
    }

    /// Select the next selectable component, wrapping around.
    /// With no selection, selects the first selectable component.
    /// While the selected component is editing, steps its value instead
    /// and returns the action it requests.
    pub fn select_next_toggleable(&mut self) -> Action {
        if let Some(popup) = &mut self.popup {
            popup.focus_next();
            return Action::None;
        }
        if let Some(index) = self.editing_index() {
            return self.components[index].edit_step();
        }
        let len = self.components.len();
        let start = self.current_toggle_index.map_or(0, |current| current + 1);
        let next = (0..len)
            .map(|offset| (start + offset) % len)
            .find(|&index| self.components[index].is_selectable());
        self.select(next);
        Action::None
    }

    /// Select the previous selectable component, wrapping around.
    /// With no selection, selects the last selectable component.
    /// Does nothing while the selected component is editing: values only step
    /// forward, as the device has no previous button.
    pub fn select_prev_toggleable(&mut self) {
        if let Some(popup) = &mut self.popup {
            popup.focus_prev();
            return;
        }
        if self.editing_index().is_some() {
            return;
        }
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
    /// Clicking a value field or a dropdown starts or ends editing it.
    /// With a popup shown, clicks its focused button, closes it and redraws the scene.
    pub fn click_currently_toggled(&mut self) -> Action {
        if let Some(mut popup) = self.popup.take() {
            self.activate();
            return popup.click();
        }
        let Some(index) = self.current_toggle_index else {
            return Action::None;
        };
        let component = &mut self.components[index];
        let was_editing = component.is_editing();
        let action = component.click();
        if was_editing && !component.is_editing() && component.edit_covers_others() {
            // The edit drew over other components: draw them again
            self.activate();
        }
        action
    }

    /// Updates the components and the popup. Returns the action of a popup that
    /// closed because its auto-dismiss timeout ran out, else `Action::None`.
    pub fn update(&mut self, state: &State) -> Action {
        for component in &mut self.components {
            component.update(state);
        }
        if let Some(popup) = &mut self.popup {
            popup.update(state);
            if let Some(action) = popup.timed_out(state.uptime_ms) {
                self.popup = None;
                self.activate();
                return action;
            }
        }
        Action::None
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

    fn popup_scene() -> Scene {
        use crate::components::popup::PopupIcon;
        let mut scene = mixed_scene();
        scene.select_next_toggleable();
        let popup = Popup::new(Size::new(320, 240), "Title", "Text", PopupIcon::Info, Action::SwitchScene(1))
            .with_dismiss(Action::SwitchScene(2), "Cancel");
        scene.show_popup(popup);
        scene
    }

    #[test]
    fn test_popup_takes_selection_and_click() {
        let mut scene = popup_scene();
        scene.select_next_toggleable();
        assert_eq!(scene.current_toggle_index, Some(1), "scene selection does not move");
        assert_eq!(scene.click_currently_toggled(), Action::SwitchScene(2));
        assert!(!scene.has_popup());
        assert_eq!(scene.click_currently_toggled(), Action::None, "button 1 has no callback");
    }

    #[test]
    fn test_popup_hides_components_until_closed() {
        let mut scene = Scene::new();
        scene.add_component(depth_label());
        let mut target = PixelCounter(0);
        scene.draw(&mut target).unwrap();

        scene.show_popup(Popup::new(
            Size::new(320, 240), "Title", "Text", crate::components::popup::PopupIcon::Info, Action::None,
        ));
        let mut state = State::new();
        state.pressure = 3.0;
        scene.update(&state);
        target.0 = 0;
        scene.draw(&mut target).unwrap();
        let popup_pixels = target.0;
        assert!(popup_pixels > 0);

        target.0 = 0;
        scene.draw(&mut target).unwrap();
        assert_eq!(target.0, 0, "the label under the popup is not drawn");

        assert_eq!(scene.click_currently_toggled(), Action::None);
        scene.draw(&mut target).unwrap();
        assert!(target.0 > 0, "closing the popup redraws the scene");
    }

    #[test]
    fn test_popup_auto_dismiss_closes_it() {
        use crate::components::popup::PopupIcon;
        let mut scene = mixed_scene();
        let popup = Popup::new(Size::new(320, 240), "Title", "Text", PopupIcon::Info, Action::None)
            .with_dismiss(Action::SwitchScene(2), "Cancel")
            .with_auto_dismiss_after(1_000);
        scene.show_popup(popup);

        let mut state = State::new();
        state.uptime_ms = 100;
        assert_eq!(scene.update(&state), Action::None);
        state.uptime_ms = 1_100;
        assert_eq!(scene.update(&state), Action::SwitchScene(2));
        assert!(!scene.has_popup());
    }

    fn settings_scene() -> Scene {
        use crate::common::Setting;
        use crate::components::dropdown::Dropdown;
        use crate::components::value_field::ValueField;
        let mut scene = Scene::new();
        scene.add_component(Drawable::ValueField(ValueField::new(Point::zero(), Size::new(10, 10), Setting::Hour)));
        scene.add_component(Drawable::Dropdown(Dropdown::new(
            Point::new(0, 20), Size::new(10, 10), Setting::UsbMode, &["A", "B"], |_| 0,
        )));
        scene.add_component(depth_label());
        scene.select_next_toggleable();
        scene
    }

    #[test]
    fn test_editing_field_takes_next() {
        use crate::common::Setting;
        let mut scene = settings_scene();
        scene.click_currently_toggled();
        assert_eq!(scene.select_next_toggleable(), Action::AdjustSetting(Setting::Hour, 1));
        scene.select_prev_toggleable();
        assert_eq!(scene.current_toggle_index, Some(0), "selection does not move while editing");

        scene.click_currently_toggled();
        assert_eq!(scene.select_next_toggleable(), Action::None);
        assert_eq!(scene.current_toggle_index, Some(1));
    }

    #[test]
    fn test_closing_dropdown_redraws_scene() {
        use crate::common::Setting;
        let mut scene = settings_scene();
        scene.select_next_toggleable();
        let mut target = PixelCounter(0);
        scene.draw(&mut target).unwrap();

        scene.click_currently_toggled();
        scene.select_next_toggleable();
        scene.draw(&mut target).unwrap();
        assert_eq!(scene.click_currently_toggled(), Action::SelectSetting(Setting::UsbMode, 1));
        assert!(scene.components[2].needs_redraw(), "the label under the list is drawn again");
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
