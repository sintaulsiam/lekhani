//! Lekhani GUI Typing Stats & OSD Helpers

use crate::{HeatmapEntry, StandaloneDialogWindow, TopBarWindow};
use lekhani_core::UserStats;
use lekhani_settings::ConfigManager;
use slint::ComponentHandle;
use std::cell::RefCell;
use std::rc::Rc;

pub fn refresh_stats_ui(app: &TopBarWindow, config_mgr: &ConfigManager) {
    let user_stats_path = config_mgr.get_user_stats_path();
    let stats = UserStats::load_from_path(&user_stats_path);
    app.set_stats_total_words(stats.total_words_typed as i32);
    app.set_stats_keystrokes_saved(stats.keystrokes_saved as i32);
    app.set_stats_efficiency_ratio(format!("{:.1}%", stats.savings_percentage()).into());

    let top_chars = stats.get_top_characters(10);
    let slint_top_chars: Vec<HeatmapEntry> = top_chars
        .into_iter()
        .map(|(ch, count, pct)| HeatmapEntry {
            character: ch.to_string().into(),
            count: count as i32,
            percentage: format!("{:.1}%", pct).into(),
        })
        .collect();
    app.set_stats_top_chars(Rc::new(slint::VecModel::from(slint_top_chars)).into());
}

pub fn refresh_standalone_stats_ui(
    standalone: &StandaloneDialogWindow,
    config_mgr: &ConfigManager,
) {
    let user_stats_path = config_mgr.get_user_stats_path();
    let stats = UserStats::load_from_path(&user_stats_path);
    standalone.set_stats_total_words(stats.total_words_typed as i32);
    standalone.set_stats_keystrokes_saved(stats.keystrokes_saved as i32);

    let top_chars = stats.get_top_characters(10);
    let slint_top_chars: Vec<HeatmapEntry> = top_chars
        .into_iter()
        .map(|(ch, count, pct)| HeatmapEntry {
            character: ch.to_string().into(),
            count: count as i32,
            percentage: format!("{:.1}%", pct).into(),
        })
        .collect();
    standalone.set_stats_top_characters(Rc::new(slint::VecModel::from(slint_top_chars)).into());
}

pub fn show_mode_osd(
    app: &TopBarWindow,
    timer: &Rc<RefCell<slint::Timer>>,
    layout_name: &str,
    is_bengali: bool,
) {
    app.set_osd_title(layout_name.into());
    if is_bengali {
        app.set_osd_subtitle("বাংলা মোড সক্রিয় (Bengali Active)".into());
        app.set_osd_icon("🇧🇩".into());
        app.set_osd_badge_color(slint::Color::from_rgb_u8(166, 227, 161)); // #a6e3a1 Green
        app.set_osd_badge_bg(slint::Color::from_argb_u8(40, 166, 227, 161));
    } else {
        app.set_osd_subtitle("English Mode Active (F12 to switch)".into());
        app.set_osd_icon("🌐".into());
        app.set_osd_badge_color(slint::Color::from_rgb_u8(137, 180, 250)); // #89b4fa Blue
        app.set_osd_badge_bg(slint::Color::from_argb_u8(40, 137, 180, 250));
    }
    app.set_show_osd(true);

    let app_weak = app.as_weak();
    timer.borrow_mut().start(
        slint::TimerMode::SingleShot,
        std::time::Duration::from_millis(1800),
        move || {
            if let Some(app) = app_weak.upgrade() {
                app.set_show_osd(false);
            }
        },
    );
}
