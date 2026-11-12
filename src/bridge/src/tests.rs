// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
use super::*;

#[test]
fn abi_layout_and_validation() {
    assert_eq!(size_of::<RuneSettings>(), 32);
    assert_eq!(align_of::<RuneSettings>(), 4);
    assert_eq!(std::mem::offset_of!(RuneSettings, speed), 24);
    let defaults = RuneSettings::from(&settings::Settings::default());
    assert!(defaults.validated().is_ok());
    let mut invalid = defaults;
    invalid.speed = f32::NAN;
    assert!(invalid.validated().is_err());
    invalid = defaults;
    invalid.accent = 6;
    assert!(invalid.validated().is_err());
}

#[test]
fn invalid_ffi_calls_do_not_start_a_window_and_errors_are_bounded() {
    // SAFETY: null is explicitly accepted for validation; invalid points to a full struct.
    unsafe {
        assert_eq!(rune_settings_default(std::ptr::null_mut()), -1);
        let required = rune_pas_last_error(std::ptr::null_mut(), 0);
        assert!(required > 1);
        let mut buffer = [42i8; 2];
        assert_eq!(rune_pas_last_error(buffer.as_mut_ptr(), 2), required);
        assert_eq!(buffer[1], 0);
        let mut invalid = RuneSettings::from(&settings::Settings::default());
        invalid.struct_size = 4;
        assert_eq!(rune_settings_run(&invalid, None, std::ptr::null_mut()), -1);
        invalid.struct_size = 32;
        invalid.menu_bar = 2;
        assert_eq!(rune_settings_run(&invalid, None, std::ptr::null_mut()), -1);
        assert!(!CONSUMED.load(Ordering::SeqCst));
    }
}
