//! Lekhani Settings UI Synchronization Helpers

use crate::{StandaloneDialogWindow, TopBarWindow};
use lekhani_settings::AppConfig;

pub fn apply_settings_to_app(app: &TopBarWindow, config: &AppConfig) {
    app.set_set_use_dict(config.phonetic.use_dictionary);
    app.set_set_include_eng(config.phonetic.include_english);
    app.set_set_enter_closes(config.phonetic.enter_key_closes_candidate_window);
    app.set_set_predictive_next(config.phonetic.enable_predictive_next_words);
    app.set_set_code_shield(config.phonetic.enable_code_shield);
    app.set_set_word_segmentation(config.phonetic.enable_word_segmentation);
    app.set_set_colloquial_dialects(config.phonetic.enable_colloquial_dialects);
    app.set_set_banglish_shorthand(config.phonetic.enable_banglish_shorthand);
    app.set_set_reduplication(config.phonetic.enable_reduplication);
    app.set_set_phrase_prediction(config.phonetic.enable_phrase_prediction);
    app.set_set_dynamic_macros(config.phonetic.enable_dynamic_macros);
    app.set_set_auto_vowel(config.fixed.auto_vowel_forming);
    app.set_set_auto_chandra(config.fixed.auto_chandra_position);
    app.set_set_traditional_kar(config.fixed.traditional_kar);
    app.set_set_old_reph(config.fixed.old_reph);
    app.set_set_numberpad(config.fixed.numberpad);
    app.set_set_toggle_key(config.general.toggle_key.clone().into());
    app.set_set_show_osd(config.general.show_osd);
    app.set_set_auto_dari(config.general.auto_dari);
    app.set_set_horizontal_cands(config.ui.horizontal_candidates);
}

pub fn apply_settings_to_standalone(s: &StandaloneDialogWindow, config: &AppConfig) {
    s.set_set_use_dict(config.phonetic.use_dictionary);
    s.set_set_include_eng(config.phonetic.include_english);
    s.set_set_enter_closes(config.phonetic.enter_key_closes_candidate_window);
    s.set_set_predictive_next(config.phonetic.enable_predictive_next_words);
    s.set_set_code_shield(config.phonetic.enable_code_shield);
    s.set_set_word_segmentation(config.phonetic.enable_word_segmentation);
    s.set_set_colloquial_dialects(config.phonetic.enable_colloquial_dialects);
    s.set_set_banglish_shorthand(config.phonetic.enable_banglish_shorthand);
    s.set_set_reduplication(config.phonetic.enable_reduplication);
    s.set_set_phrase_prediction(config.phonetic.enable_phrase_prediction);
    s.set_set_dynamic_macros(config.phonetic.enable_dynamic_macros);
    s.set_set_auto_vowel(config.fixed.auto_vowel_forming);
    s.set_set_auto_chandra(config.fixed.auto_chandra_position);
    s.set_set_traditional_kar(config.fixed.traditional_kar);
    s.set_set_old_reph(config.fixed.old_reph);
    s.set_set_numberpad(config.fixed.numberpad);
    s.set_set_toggle_key(config.general.toggle_key.clone().into());
    s.set_set_show_osd(config.general.show_osd);
    s.set_set_auto_dari(config.general.auto_dari);
    s.set_set_horizontal_cands(config.ui.horizontal_candidates);
}

pub fn read_settings_from_app(app: &TopBarWindow, config: &mut AppConfig) {
    config.phonetic.use_dictionary = app.get_set_use_dict();
    config.phonetic.include_english = app.get_set_include_eng();
    config.phonetic.enter_key_closes_candidate_window = app.get_set_enter_closes();
    config.phonetic.enable_predictive_next_words = app.get_set_predictive_next();
    config.phonetic.enable_code_shield = app.get_set_code_shield();
    config.phonetic.enable_word_segmentation = app.get_set_word_segmentation();
    config.phonetic.enable_colloquial_dialects = app.get_set_colloquial_dialects();
    config.phonetic.enable_banglish_shorthand = app.get_set_banglish_shorthand();
    config.phonetic.enable_reduplication = app.get_set_reduplication();
    config.phonetic.enable_phrase_prediction = app.get_set_phrase_prediction();
    config.phonetic.enable_dynamic_macros = app.get_set_dynamic_macros();
    config.fixed.auto_vowel_forming = app.get_set_auto_vowel();
    config.fixed.auto_chandra_position = app.get_set_auto_chandra();
    config.fixed.traditional_kar = app.get_set_traditional_kar();
    config.fixed.old_reph = app.get_set_old_reph();
    config.fixed.numberpad = app.get_set_numberpad();
    config.general.toggle_key = app.get_set_toggle_key().to_string();
    config.general.show_osd = app.get_set_show_osd();
    config.general.auto_dari = app.get_set_auto_dari();
    config.ui.horizontal_candidates = app.get_set_horizontal_cands();
}

pub fn read_settings_from_standalone(s: &StandaloneDialogWindow, config: &mut AppConfig) {
    config.phonetic.use_dictionary = s.get_set_use_dict();
    config.phonetic.include_english = s.get_set_include_eng();
    config.phonetic.enter_key_closes_candidate_window = s.get_set_enter_closes();
    config.phonetic.enable_predictive_next_words = s.get_set_predictive_next();
    config.phonetic.enable_code_shield = s.get_set_code_shield();
    config.phonetic.enable_word_segmentation = s.get_set_word_segmentation();
    config.phonetic.enable_colloquial_dialects = s.get_set_colloquial_dialects();
    config.phonetic.enable_banglish_shorthand = s.get_set_banglish_shorthand();
    config.phonetic.enable_reduplication = s.get_set_reduplication();
    config.phonetic.enable_phrase_prediction = s.get_set_phrase_prediction();
    config.phonetic.enable_dynamic_macros = s.get_set_dynamic_macros();
    config.fixed.auto_vowel_forming = s.get_set_auto_vowel();
    config.fixed.auto_chandra_position = s.get_set_auto_chandra();
    config.fixed.traditional_kar = s.get_set_traditional_kar();
    config.fixed.old_reph = s.get_set_old_reph();
    config.fixed.numberpad = s.get_set_numberpad();
    config.general.toggle_key = s.get_set_toggle_key().to_string();
    config.general.show_osd = s.get_set_show_osd();
    config.general.auto_dari = s.get_set_auto_dari();
    config.ui.horizontal_candidates = s.get_set_horizontal_cands();
}
