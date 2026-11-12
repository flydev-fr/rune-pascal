// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
use super::*;

#[test]
fn all_scenes_fit_the_abi_and_validate() {
    assert_eq!(size_of::<RuneLabConfig>(), 80);
    assert_eq!(align_of::<RuneLabConfig>(), 4);
    assert_eq!(size_of::<RuneLabKnobInfo>(), 116);
    assert_eq!(std::mem::offset_of!(RuneLabKnobInfo, name), 20);
    for scene in 0..ALL.len() as u32 {
        let mut s = lab::Lab::new();
        let c = RuneLabConfig { scene, ..RuneLabConfig::default() };
        assert!(c.validate().is_ok());
        c.apply(&mut s);
        let mut snap = RuneLabConfig::snapshot(&s);
        assert_eq!(snap.scene, scene);
        assert!(snap.validate().is_ok());
        snap.knob_values[0] = f32::NAN;
        assert!(snap.validate().is_err());
    }
}

#[test]
fn invalid_configs_and_unstarted_controls_do_not_consume_event_loop() {
    let mut c = RuneLabConfig::default();
    // SAFETY: null arguments are explicitly validated, and local structs are valid storage.
    unsafe {
        assert_eq!(rune_lab_default(std::ptr::null_mut()), -1);
        assert_eq!(rune_lab_knob_info(99, 0, std::ptr::null_mut()), -1);
        assert_eq!(rune_lab_set_config(&c), -5);
        assert_eq!(rune_lab_quit(), -5);
        c.scene = 99;
        assert_eq!(rune_lab_run(&c, None, std::ptr::null_mut()), -1);
        c.scene = 0;
        c.knob_count = 16;
        assert_eq!(rune_lab_run(&c, None, std::ptr::null_mut()), -1);
        c.knob_count = 0;
        c.struct_size = 4;
        assert_eq!(rune_lab_run(&c, None, std::ptr::null_mut()), -1);
    }
    assert!(!CONSUMED.load(Ordering::SeqCst));
}
