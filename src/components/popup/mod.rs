use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::{MonoFont, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};
use embedded_graphics::geometry::Size;
use profont::{PROFONT_12_POINT, PROFONT_18_POINT, PROFONT_9_POINT};

use crate::common::{copy_truncated, Action, Clickable, DrawableComponent, State, Toggleable};
use crate::components::button::Button;
use heapless::String;
use core::option::Option::Some;
use core::option::Option::None;
use core::result::Result;
use core::result::Result::Ok;
use core::option::Option;

const MAX_TITLE_LENGTH: usize = 32;
const MAX_TEXT_LENGTH: usize = 160;

/// How long an auto-dismissing popup stays open unless told otherwise.
pub const DEFAULT_AUTO_DISMISS_MS: u32 = 5_000;

/// Space between the screen edge and a popup of the default size.
const SCREEN_MARGIN: u32 = 16;
const TITLE_BAR_HEIGHT: u32 = 20;
const PADDING: i32 = 8;
const ICON_SIZE: u32 = 32;
/// Narrowest button. Buttons widen to fit longer labels.
const BUTTON_WIDTH: u32 = 80;
const BUTTON_HEIGHT: u32 = 24;
const BUTTON_GAP: i32 = 16;
const LINE_SPACING: u32 = 2;

const TEXT_FONT: &MonoFont<'static> = &PROFONT_9_POINT;
/// The font `Button` draws its label with.
const BUTTON_FONT: &MonoFont<'static> = &PROFONT_9_POINT;
const BACKGROUND: Rgb565 = Rgb565::BLACK;
const TITLE_BAR: Rgb565 = Rgb565::new(10, 20, 10);

/// The symbol drawn on the left of the popup text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupIcon {
    Info,
    Warning,
    Error,
}

/// The popup button that has the focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Choice {
    Accept,
    Dismiss,
}

/// A modal box with a title, an icon, some text, an accept button and an optional
/// dismiss button. It is centred on the screen whatever its size.
///
/// Clicking a button returns that button's action. The popup is then done and its
/// owner removes it: see `Scene::show_popup`. To only close the popup on accept,
/// give `Action::None` as the accept action.
pub struct Popup {
    screen_size: Size,
    area: Rectangle,
    title: String<MAX_TITLE_LENGTH>,
    text: String<MAX_TEXT_LENGTH>,
    icon: PopupIcon,
    accept_action: Action,
    accept_button: Button,
    /// The dismiss action and its button. `None` shows the accept button only.
    dismiss: Option<(Action, Button)>,
    focus: Choice,
    needs_redraw: bool,
    /// Time after which the popup closes by itself. `None` waits for a click.
    auto_dismiss_ms: Option<u32>,
    /// Uptime at the first update after the popup was shown.
    shown_at_ms: Option<u32>,
}

impl Popup {
    /// Creates a popup centred on a screen of `screen_size`, with an "OK" button
    /// that returns `accept_action`. It fills the screen except for a margin,
    /// until `with_size` changes that. Title and text that do not fit are truncated.
    pub fn new(screen_size: Size, title: &str, text: &str, icon: PopupIcon, accept_action: Action) -> Self {
        let size = Size::new(
            screen_size.width.saturating_sub(2 * SCREEN_MARGIN),
            screen_size.height.saturating_sub(2 * SCREEN_MARGIN),
        );
        let mut title_str = String::new();
        copy_truncated(&mut title_str, title);
        let mut text_str = String::new();
        copy_truncated(&mut text_str, text);

        let mut popup = Self {
            screen_size,
            area: centered(screen_size, size),
            title: title_str,
            text: text_str,
            icon,
            accept_action,
            accept_button: Button::new(Point::zero(), Size::new(BUTTON_WIDTH, BUTTON_HEIGHT), "OK"),
            dismiss: None,
            focus: Choice::Accept,
            needs_redraw: true,
            auto_dismiss_ms: None,
            shown_at_ms: None,
        };
        popup.layout_buttons();
        popup.accept_button.toggle(true);
        popup
    }

    /// Sets the popup size, keeping it centred. A size bigger than the screen is
    /// reduced to the screen size.
    pub fn with_size(mut self, size: Size) -> Self {
        self.area = centered(self.screen_size, size.component_min(self.screen_size));
        self.layout_buttons();
        self
    }

    /// Sets the accept button label, "OK" by default.
    pub fn with_accept_label(mut self, label: &str) -> Self {
        copy_truncated(&mut self.accept_button.label, label);
        self.layout_buttons();
        self
    }

    /// Adds a dismiss button, labelled `label`, that returns `dismiss_action`.
    pub fn with_dismiss(mut self, dismiss_action: Action, label: &str) -> Self {
        let button = Button::new(Point::zero(), Size::new(BUTTON_WIDTH, BUTTON_HEIGHT), label);
        self.dismiss = Some((dismiss_action, button));
        self.layout_buttons();
        self
    }

    /// Closes the popup by itself `DEFAULT_AUTO_DISMISS_MS` after it is shown.
    pub fn with_auto_dismiss(self) -> Self {
        self.with_auto_dismiss_after(DEFAULT_AUTO_DISMISS_MS)
    }

    /// Closes the popup by itself `timeout_ms` after it is shown.
    /// Closing this way returns the dismiss action, or `Action::None` without a
    /// dismiss button: the accept action only runs when someone accepts.
    pub fn with_auto_dismiss_after(mut self, timeout_ms: u32) -> Self {
        self.auto_dismiss_ms = Some(timeout_ms);
        self
    }

    /// Returns the action to run if the auto-dismiss timeout ran out at `uptime_ms`.
    /// The time counts from the first `update` after the popup was shown.
    pub fn timed_out(&self, uptime_ms: u32) -> Option<Action> {
        let timeout = self.auto_dismiss_ms?;
        let shown_at = self.shown_at_ms?;
        (uptime_ms.wrapping_sub(shown_at) >= timeout)
            .then(|| self.dismiss.as_ref().map_or(Action::None, |(action, _)| *action))
    }

    /// Sizes the buttons to their labels and places them at the bottom of the
    /// popup: accept alone in the middle, or dismiss on the left and accept on the right.
    fn layout_buttons(&mut self) {
        let center_x = self.area.top_left.x + self.area.size.width as i32 / 2;
        let y = self.area.top_left.y + self.area.size.height as i32 - PADDING - BUTTON_HEIGHT as i32;
        self.accept_button.size = button_size(&self.accept_button.label);
        let accept_width = self.accept_button.size.width as i32;
        match &mut self.dismiss {
            Some((_, dismiss_button)) => {
                dismiss_button.size = button_size(&dismiss_button.label);
                let dismiss_width = dismiss_button.size.width as i32;
                // The gap between the buttons sits in the middle of the pair
                let left = center_x - (dismiss_width + BUTTON_GAP + accept_width) / 2;
                dismiss_button.position = Point::new(left, y);
                self.accept_button.position = Point::new(left + dismiss_width + BUTTON_GAP, y);
            }
            None => self.accept_button.position = Point::new(center_x - accept_width / 2, y),
        }
    }

    /// Moves the focus to the other button. Does nothing without a dismiss button.
    pub fn focus_next(&mut self) {
        let Some((_, dismiss_button)) = &mut self.dismiss else {
            return;
        };
        self.focus = match self.focus {
            Choice::Accept => Choice::Dismiss,
            Choice::Dismiss => Choice::Accept,
        };
        self.accept_button.toggle(self.focus == Choice::Accept);
        dismiss_button.toggle(self.focus == Choice::Dismiss);
        self.needs_redraw = true;
    }

    /// With two buttons, moving back is the same as moving forward.
    pub fn focus_prev(&mut self) {
        self.focus_next();
    }

    /// Draws the popup on the given draw target.
    pub fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        let frame = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb565::WHITE)
            .stroke_width(2)
            .stroke_alignment(StrokeAlignment::Inside)
            .fill_color(BACKGROUND)
            .build();
        self.area.into_styled(frame).draw(target)?;

        // Title bar, inside the border
        let top_left = self.area.top_left;
        let title_bar = Rectangle::new(
            top_left + Point::new(2, 2),
            Size::new(self.area.size.width.saturating_sub(4), TITLE_BAR_HEIGHT),
        );
        title_bar.into_styled(PrimitiveStyle::with_fill(TITLE_BAR)).draw(target)?;
        Text::with_text_style(
            &self.title,
            title_bar.center(),
            MonoTextStyle::new(&PROFONT_12_POINT, Rgb565::WHITE),
            TextStyleBuilder::new().alignment(Alignment::Center).baseline(Baseline::Middle).build(),
        )
        .draw(target)?;

        let body_top = title_bar.top_left.y + TITLE_BAR_HEIGHT as i32 + PADDING;
        let icon_area = Rectangle::new(
            Point::new(top_left.x + PADDING, body_top),
            Size::new(ICON_SIZE, ICON_SIZE),
        );
        draw_icon(self.icon, icon_area, target)?;

        // Text, wrapped to the space between the icon and the right border,
        // and cut off above the buttons
        let text_left = icon_area.top_left.x + ICON_SIZE as i32 + PADDING;
        let text_right = top_left.x + self.area.size.width as i32 - PADDING;
        let buttons_top = self.accept_button.position.y - PADDING;
        let char_width = TEXT_FONT.character_size.width + TEXT_FONT.character_spacing;
        let line_height = (TEXT_FONT.character_size.height + LINE_SPACING) as i32;
        let max_chars = ((text_right - text_left).max(0) as u32 / char_width) as usize;
        let text_style = MonoTextStyle::new(TEXT_FONT, Rgb565::WHITE);
        let mut y = body_top;
        for line in wrap(&self.text, max_chars) {
            if y + line_height > buttons_top {
                break;
            }
            Text::with_baseline(line, Point::new(text_left, y), text_style, Baseline::Top).draw(target)?;
            y += line_height;
        }

        if let Some((_, dismiss_button)) = &mut self.dismiss {
            dismiss_button.draw(target)?;
        }
        self.accept_button.draw(target)
    }
}

/// A rectangle of `size` centred on a screen of `screen_size`.
fn centered(screen_size: Size, size: Size) -> Rectangle {
    let top_left = Point::new(
        (screen_size.width as i32 - size.width as i32) / 2,
        (screen_size.height as i32 - size.height as i32) / 2,
    );
    Rectangle::new(top_left, size)
}

/// The size of a button that fits `label` with some padding, and at least `BUTTON_WIDTH` wide.
fn button_size(label: &str) -> Size {
    let char_width = BUTTON_FONT.character_size.width + BUTTON_FONT.character_spacing;
    let text_width = label.chars().count() as u32 * char_width;
    Size::new((text_width + 2 * PADDING as u32).max(BUTTON_WIDTH), BUTTON_HEIGHT)
}

/// Draws `icon` filling `area`: a blue "i" disc, a yellow "!" triangle or a red "X" disc.
fn draw_icon<D: DrawTarget<Color = Rgb565>>(icon: PopupIcon, area: Rectangle, target: &mut D) -> Result<(), D::Error> {
    let (color, symbol, symbol_color) = match icon {
        PopupIcon::Info => (Rgb565::BLUE, "i", Rgb565::WHITE),
        PopupIcon::Warning => (Rgb565::YELLOW, "!", Rgb565::BLACK),
        PopupIcon::Error => (Rgb565::RED, "X", Rgb565::WHITE),
    };
    let fill = PrimitiveStyle::with_fill(color);
    let mut symbol_center = area.center();
    match icon {
        PopupIcon::Warning => {
            let bottom = area.top_left.y + area.size.height as i32 - 1;
            let right = area.top_left.x + area.size.width as i32 - 1;
            Triangle::new(
                Point::new(area.center().x, area.top_left.y),
                Point::new(right, bottom),
                Point::new(area.top_left.x, bottom),
            )
            .into_styled(fill)
            .draw(target)?;
            // The triangle is wider at the bottom, so the symbol sits lower
            symbol_center.y += area.size.height as i32 / 6;
        }
        PopupIcon::Info | PopupIcon::Error => {
            Circle::new(area.top_left, area.size.width).into_styled(fill).draw(target)?;
        }
    }
    Text::with_text_style(
        symbol,
        symbol_center,
        MonoTextStyle::new(&PROFONT_18_POINT, symbol_color),
        TextStyleBuilder::new().alignment(Alignment::Center).baseline(Baseline::Middle).build(),
    )
    .draw(target)?;
    Ok(())
}

/// Splits `text` into lines of at most `max_chars` characters.
/// Breaks at spaces where possible, at `\n` always, and inside words longer than a line.
fn wrap(text: &str, max_chars: usize) -> impl Iterator<Item = &str> {
    let max_chars = max_chars.max(1);
    text.split('\n').flat_map(move |paragraph| {
        let mut rest = paragraph.trim_start();
        core::iter::from_fn(move || {
            if rest.is_empty() {
                return None;
            }
            // Byte offset just past `max_chars` characters, or the whole rest
            let limit = rest.char_indices().nth(max_chars).map_or(rest.len(), |(i, _)| i);
            let end = if limit == rest.len() {
                limit
            } else if rest[limit..].starts_with(' ') {
                // The line ends exactly at a word boundary
                limit
            } else {
                // Break at the last space within the limit
                match rest[..limit].rfind(' ') {
                    Some(space) if space > 0 => space,
                    _ => limit,
                }
            };
            let line = rest[..end].trim_end();
            rest = rest[end..].trim_start();
            Some(line)
        })
    })
}

impl DrawableComponent for Popup {
    fn initialize(&mut self) {
        self.needs_redraw = true;
    }

    fn draw<D: DrawTarget<Color = Rgb565>>(&mut self, target: &mut D) -> Result<(), D::Error> {
        self.draw(target)
    }

    fn update(&mut self, state: &State) {
        if self.shown_at_ms.is_none() {
            self.shown_at_ms = Some(state.uptime_ms);
        }
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn set_needs_redraw(&mut self, needs_redraw: bool) {
        self.needs_redraw = needs_redraw;
    }
}

impl Clickable for Popup {
    /// Returns the action of the focused button.
    fn click(&mut self) -> Action {
        match (self.focus, &self.dismiss) {
            (Choice::Dismiss, Some((dismiss_action, _))) => *dismiss_action,
            _ => self.accept_action,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::Infallible;
    use heapless::Vec;

    fn state_at(uptime_ms: u32) -> State {
        let mut state = State::new();
        state.uptime_ms = uptime_ms;
        state
    }

    #[test]
    fn popup_without_auto_dismiss_never_times_out() {
        let mut popup = popup();
        popup.update(&state_at(1_000));
        assert_eq!(popup.timed_out(1_000_000), None);
    }

    #[test]
    fn auto_dismiss_defaults_to_five_seconds_from_first_update() {
        let mut popup = popup().with_auto_dismiss();
        assert_eq!(popup.timed_out(100_000), None, "the clock starts at the first update");
        popup.update(&state_at(10_000));
        popup.update(&state_at(12_000));
        assert_eq!(popup.timed_out(14_999), None);
        assert_eq!(popup.timed_out(15_000), Some(Action::None));
    }

    #[test]
    fn auto_dismiss_returns_dismiss_action_with_custom_timeout() {
        let mut popup = popup().with_dismiss(Action::SwitchScene(2), "Cancel").with_auto_dismiss_after(500);
        popup.update(&state_at(0));
        assert_eq!(popup.timed_out(499), None);
        assert_eq!(popup.timed_out(500), Some(Action::SwitchScene(2)));
    }

    fn popup() -> Popup {
        Popup::new(Size::new(320, 240), "Title", "Some text", PopupIcon::Info, Action::SwitchScene(1))
    }

    fn lines(text: &str, max_chars: usize) -> Vec<&str, 16> {
        wrap(text, max_chars).collect()
    }

    #[test]
    fn wrap_breaks_at_spaces() {
        assert_eq!(lines("the quick brown fox", 10).as_slice(), ["the quick", "brown fox"]);
    }

    #[test]
    fn wrap_keeps_a_word_that_ends_on_the_limit() {
        assert_eq!(lines("abcde fgh", 5).as_slice(), ["abcde", "fgh"]);
    }

    #[test]
    fn wrap_splits_long_words() {
        assert_eq!(lines("abcdefgh", 3).as_slice(), ["abc", "def", "gh"]);
    }

    #[test]
    fn wrap_breaks_at_newlines() {
        assert_eq!(lines("one\ntwo", 20).as_slice(), ["one", "two"]);
    }

    #[test]
    fn accept_only_popup_returns_accept_action() {
        let mut popup = popup();
        popup.focus_next();
        assert_eq!(popup.click(), Action::SwitchScene(1));
    }

    #[test]
    fn focus_moves_between_buttons() {
        let mut popup = popup().with_dismiss(Action::SwitchScene(2), "Cancel");
        assert_eq!(popup.click(), Action::SwitchScene(1));
        popup.focus_next();
        assert_eq!(popup.click(), Action::SwitchScene(2));
        popup.focus_prev();
        assert_eq!(popup.click(), Action::SwitchScene(1));
    }

    #[test]
    fn buttons_do_not_overlap() {
        let popup = popup().with_dismiss(Action::None, "Cancel");
        let (_, dismiss_button) = popup.dismiss.as_ref().unwrap();
        assert!(dismiss_button.position.x + (BUTTON_WIDTH as i32) < popup.accept_button.position.x);
    }

    #[test]
    fn labels_are_configurable_and_buttons_fit_them() {
        let popup = popup()
            .with_accept_label("Acknowledge warning")
            .with_dismiss(Action::None, "No");
        let (_, dismiss_button) = popup.dismiss.as_ref().unwrap();
        assert_eq!(popup.accept_button.label.as_str(), "Acknowledge warning");
        assert_eq!(dismiss_button.label.as_str(), "No");
        assert!(popup.accept_button.size.width > BUTTON_WIDTH);
        assert_eq!(dismiss_button.size.width, BUTTON_WIDTH);
        // The pair stays centred on the popup
        let left_space = dismiss_button.position.x - popup.area.top_left.x;
        let right_edge = popup.accept_button.position.x + popup.accept_button.size.width as i32;
        let right_space = popup.area.top_left.x + popup.area.size.width as i32 - right_edge;
        assert!((left_space - right_space).abs() <= 1);
    }

    #[test]
    fn custom_size_stays_centred() {
        let popup = popup().with_size(Size::new(200, 120));
        assert_eq!(popup.area, Rectangle::new(Point::new(60, 60), Size::new(200, 120)));
    }

    #[test]
    fn size_is_limited_to_the_screen() {
        let popup = popup().with_size(Size::new(400, 100));
        assert_eq!(popup.area, Rectangle::new(Point::new(0, 70), Size::new(320, 100)));
    }

    /// Records the area the drawn pixels cover.
    struct Extent(Option<(Point, Point)>);

    impl OriginDimensions for Extent {
        fn size(&self) -> Size {
            Size::new(320, 240)
        }
    }

    impl DrawTarget for Extent {
        type Color = Rgb565;
        type Error = Infallible;

        fn draw_iter<I: IntoIterator<Item = Pixel<Rgb565>>>(&mut self, pixels: I) -> Result<(), Infallible> {
            for Pixel(p, _) in pixels {
                self.0 = Some(match self.0 {
                    None => (p, p),
                    Some((min, max)) => (min.component_min(p), max.component_max(p)),
                });
            }
            Ok(())
        }
    }

    #[test]
    fn draws_inside_its_area() {
        let long_text = "A very long text that does not fit in the popup and has to be cut off before it reaches the buttons at the bottom of the box";
        let mut popup = Popup::new(Size::new(320, 240), "Title", long_text, PopupIcon::Warning, Action::None)
            .with_dismiss(Action::None, "Cancel");
        let mut extent = Extent(None);
        popup.draw(&mut extent).unwrap();
        let (min, max) = extent.0.unwrap();
        assert_eq!(min, Point::new(16, 16));
        assert_eq!(max, Point::new(303, 223));
    }
}
