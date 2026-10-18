//! User settings and the rules for changing them from the settings scene.

use super::State;

/// Step for a gradient factor change, in percent.
pub const GF_STEP: u8 = 5;
/// Lowest gradient factor, in percent.
pub const GF_MIN: u8 = 10;
/// Highest gradient factor, in percent.
pub const GF_MAX: u8 = 100;
/// Lowest estimated SAC rate, in litres per minute.
pub const SAC_MIN: u8 = 10;
/// Highest estimated SAC rate, in litres per minute.
pub const SAC_MAX: u8 = 30;
/// Lowest year the clock can be set to.
pub const YEAR_MIN: u16 = 2026;
/// Highest year the clock can be set to.
pub const YEAR_MAX: u16 = 2036;
/// Step for a backlight change, in percent of full brightness.
pub const BACKLIGHT_STEP: u8 = 10;
/// Lowest backlight setting, in percent.
pub const BACKLIGHT_MIN: u8 = 10;
/// Highest backlight setting, in percent.
pub const BACKLIGHT_MAX: u8 = 100;
/// Idle times after which the screen turns off, in seconds, in the order the
/// settings scene steps through them. 0 keeps the screen on.
pub const SCREEN_OFF_OPTIONS_S: [u16; 6] = [15, 30, 60, 120, 300, 0];

/// What the USB port does when a cable is connected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsbMode {
    /// A serial port for logs and commands.
    Serial,
}

impl UsbMode {
    /// All modes, in the order the settings dropdown lists them.
    pub const ALL: [UsbMode; 1] = [UsbMode::Serial];
    /// The names of `ALL`, in the same order.
    pub const LABELS: [&'static str; 1] = ["Serial"];

    /// Position of this mode in `ALL`.
    pub fn index(self) -> usize {
        match self {
            UsbMode::Serial => 0,
        }
    }

    /// The mode at `index` in `ALL`, or `None` if there is no such mode.
    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    /// The name shown for this mode.
    pub fn label(self) -> &'static str {
        Self::LABELS[self.index()]
    }
}

/// Settings the diver can change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Gradient factor at the surface, in percent.
    pub gf_high: u8,
    /// Gradient factor at the first stop, in percent. Never above `gf_high`.
    pub gf_low: u8,
    /// Estimated surface air consumption, in litres per minute.
    pub sac_l_min: u8,
    pub usb_mode: UsbMode,
    /// Backlight brightness, in percent: `BACKLIGHT_MIN..=BACKLIGHT_MAX` in
    /// steps of `BACKLIGHT_STEP`.
    pub backlight_percent: u8,
    /// Idle time before the screen turns off, in seconds: one of
    /// `SCREEN_OFF_OPTIONS_S`. 0 keeps the screen on.
    pub screen_off_s: u16,
}

impl Settings {
    pub const fn new() -> Self {
        Self {
            gf_high: 70,
            gf_low: 30,
            sac_l_min: 20,
            usb_mode: UsbMode::Serial,
            backlight_percent: 10,
            screen_off_s: 30,
        }
    }
}

/// Position of `seconds` in `SCREEN_OFF_OPTIONS_S`, or `None` if it is not an option.
pub fn screen_off_index(seconds: u16) -> Option<usize> {
    SCREEN_OFF_OPTIONS_S.iter().position(|&option| option == seconds)
}

/// A value the settings scene can change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Day,
    Month,
    Year,
    Hour,
    Minute,
    GfHigh,
    GfLow,
    SacRate,
    UsbMode,
    Backlight,
    ScreenOff,
}

/// Number of days in `month` (1 to 12) of `year`.
pub fn days_in_month(month: u8, year: u16) -> u8 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `value` moved by `delta` steps of `step`, wrapped into `min..=max`.
/// A `value` outside the range or off the step grid snaps onto it first.
fn wrap(value: u16, delta: i8, min: u16, max: u16, step: u16) -> u16 {
    let count = ((max - min) / step + 1) as i32;
    let index = (value.clamp(min, max) - min) as i32 / step as i32;
    min + ((index + delta as i32).rem_euclid(count) as u16) * step
}

/// `wrap` for `u8` values.
fn wrap_u8(value: u8, delta: i8, min: u8, max: u8, step: u8) -> u8 {
    wrap(value as u16, delta, min as u16, max as u16, step as u16) as u8
}

impl State {
    /// Moves `setting` by `delta` steps.
    ///
    /// The device has only next and enter buttons, so values only go up in
    /// practice: every setting wraps from its highest value back to its lowest.
    /// Date and time fields wrap without carrying into the next field, and the
    /// day is kept valid for the month and year. GF low never goes above GF high:
    /// GF high wraps down to GF low, and GF low wraps at GF high.
    /// The USB mode cycles through `UsbMode::ALL` and the screen-off time
    /// through `SCREEN_OFF_OPTIONS_S`.
    pub fn adjust_setting(&mut self, setting: Setting, delta: i8) {
        let date = &mut self.datetime.date;
        let time = &mut self.datetime.time;
        let settings = &mut self.settings;
        match setting {
            Setting::Day => date.day = wrap_u8(date.day, delta, 1, days_in_month(date.month, date.year), 1),
            Setting::Month => date.month = wrap_u8(date.month, delta, 1, 12, 1),
            Setting::Year => date.year = wrap(date.year, delta, YEAR_MIN, YEAR_MAX, 1),
            Setting::Hour => time.hours = wrap_u8(time.hours, delta, 0, 23, 1),
            Setting::Minute => time.minutes = wrap_u8(time.minutes, delta, 0, 59, 1),
            Setting::GfHigh => {
                let min = settings.gf_low.max(GF_MIN);
                settings.gf_high = wrap_u8(settings.gf_high, delta, min, GF_MAX, GF_STEP);
            }
            Setting::GfLow => {
                settings.gf_low = wrap_u8(settings.gf_low, delta, GF_MIN, settings.gf_high, GF_STEP);
            }
            Setting::SacRate => settings.sac_l_min = wrap_u8(settings.sac_l_min, delta, SAC_MIN, SAC_MAX, 1),
            Setting::UsbMode => {
                let count = UsbMode::ALL.len() as i16;
                let index = (settings.usb_mode.index() as i16 + delta as i16).rem_euclid(count);
                settings.usb_mode = UsbMode::ALL[index as usize];
            }
            Setting::Backlight => {
                settings.backlight_percent =
                    wrap_u8(settings.backlight_percent, delta, BACKLIGHT_MIN, BACKLIGHT_MAX, BACKLIGHT_STEP);
            }
            Setting::ScreenOff => {
                let count = SCREEN_OFF_OPTIONS_S.len() as i16;
                // A time that is not an option counts as the first one
                let index = screen_off_index(settings.screen_off_s).unwrap_or(0) as i16;
                settings.screen_off_s = SCREEN_OFF_OPTIONS_S[(index + delta as i16).rem_euclid(count) as usize];
            }
        }
        // A shorter month or a non-leap year can make the day invalid
        date.day = date.day.min(days_in_month(date.month, date.year));
    }

    /// Sets a setting chosen from a list to the option at `index`.
    /// Only the USB mode is chosen this way: other settings, and an index
    /// with no option, are ignored.
    pub fn select_setting(&mut self, setting: Setting, index: usize) {
        if let (Setting::UsbMode, Some(mode)) = (setting, UsbMode::from_index(index)) {
            self.settings.usb_mode = mode;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{Date, Time};

    fn state_at(day: u8, month: u8, year: u16) -> State {
        let mut state = State::new();
        state.datetime.date = Date::new(day, month, year);
        state
    }

    #[test]
    fn test_time_wraps_without_carry() {
        let mut state = State::new();
        state.datetime.time = Time::new(23, 59);
        state.adjust_setting(Setting::Minute, 1);
        assert_eq!(state.datetime.time.minutes, 0);
        assert_eq!(state.datetime.time.hours, 23);
        state.adjust_setting(Setting::Hour, 1);
        assert_eq!(state.datetime.time.hours, 0);
        state.adjust_setting(Setting::Hour, -1);
        assert_eq!(state.datetime.time.hours, 23);
    }

    #[test]
    fn test_day_wraps_within_month() {
        let mut state = state_at(30, 4, 2026);
        state.adjust_setting(Setting::Day, 1);
        assert_eq!(state.datetime.date.day, 1);
        state.adjust_setting(Setting::Day, -1);
        assert_eq!(state.datetime.date.day, 30);
    }

    #[test]
    fn test_month_change_keeps_day_valid() {
        let mut state = state_at(31, 1, 2026);
        state.adjust_setting(Setting::Month, 1);
        assert_eq!((state.datetime.date.day, state.datetime.date.month), (28, 2));
    }

    #[test]
    fn test_year_change_keeps_leap_day_valid() {
        let mut state = state_at(29, 2, 2028);
        state.adjust_setting(Setting::Year, 1);
        assert_eq!(state.datetime.date.day, 28);
    }

    #[test]
    fn test_year_wraps_in_range() {
        let mut state = state_at(1, 1, YEAR_MAX);
        state.adjust_setting(Setting::Year, 1);
        assert_eq!(state.datetime.date.year, YEAR_MIN);
        state.datetime.date.year = 2020;
        state.adjust_setting(Setting::Year, 1);
        assert_eq!(state.datetime.date.year, YEAR_MIN + 1, "a year out of range snaps into it");
    }

    #[test]
    fn test_gradient_factors_wrap_and_stay_ordered() {
        let mut state = State::new();
        state.settings.gf_low = 65;
        state.adjust_setting(Setting::GfLow, 1);
        assert_eq!(state.settings.gf_low, 70);
        state.adjust_setting(Setting::GfLow, 1);
        assert_eq!(state.settings.gf_low, GF_MIN, "GF low wraps at GF high");

        state.settings.gf_low = 30;
        state.settings.gf_high = 95;
        state.adjust_setting(Setting::GfHigh, 1);
        assert_eq!(state.settings.gf_high, GF_MAX);
        state.adjust_setting(Setting::GfHigh, 1);
        assert_eq!(state.settings.gf_high, 30, "GF high wraps down to GF low");
    }

    #[test]
    fn test_sac_rate_wraps_in_range() {
        let mut state = State::new();
        state.settings.sac_l_min = SAC_MAX;
        state.adjust_setting(Setting::SacRate, 1);
        assert_eq!(state.settings.sac_l_min, SAC_MIN);
        state.adjust_setting(Setting::SacRate, 1);
        assert_eq!(state.settings.sac_l_min, SAC_MIN + 1);
    }

    #[test]
    fn test_backlight_wraps_in_range() {
        let mut state = State::new();
        state.settings.backlight_percent = 90;
        state.adjust_setting(Setting::Backlight, 1);
        assert_eq!(state.settings.backlight_percent, BACKLIGHT_MAX);
        state.adjust_setting(Setting::Backlight, 1);
        assert_eq!(state.settings.backlight_percent, BACKLIGHT_MIN);
    }

    #[test]
    fn test_screen_off_steps_through_options() {
        let mut state = State::new();
        state.settings.screen_off_s = 300;
        state.adjust_setting(Setting::ScreenOff, 1);
        assert_eq!(state.settings.screen_off_s, 0, "never comes after the longest time");
        state.adjust_setting(Setting::ScreenOff, 1);
        assert_eq!(state.settings.screen_off_s, SCREEN_OFF_OPTIONS_S[0], "and wraps to the shortest");
        state.settings.screen_off_s = 45;
        state.adjust_setting(Setting::ScreenOff, 1);
        assert_eq!(state.settings.screen_off_s, SCREEN_OFF_OPTIONS_S[1], "an unknown time snaps onto the options");
    }

    #[test]
    fn test_select_usb_mode() {
        let mut state = State::new();
        state.select_setting(Setting::UsbMode, 0);
        assert_eq!(state.settings.usb_mode, UsbMode::Serial);
        state.select_setting(Setting::UsbMode, 5);
        assert_eq!(state.settings.usb_mode, UsbMode::Serial, "unknown index is ignored");
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2, 2024), 29);
        assert_eq!(days_in_month(2, 2100), 28);
        assert_eq!(days_in_month(2, 2000), 29);
        assert_eq!(days_in_month(9, 2026), 30);
        assert_eq!(days_in_month(12, 2026), 31);
    }
}
