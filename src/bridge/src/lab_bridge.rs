// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
//! Lab C ABI: stable scene IDs, knob metadata and queued runtime updates.
use std::cell::Cell;
use std::ffi::{c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::{Mutex, atomic::Ordering};

use lab::knobs::{ALL, Kind, specs};
use super::{CONSUMED, ERROR, fail};

const MAX_KNOBS: usize = 16;
static ACTIVE: Mutex<Option<rune::AppHandle<lab::Lab>>> = Mutex::new(None);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuneLabConfig {
    pub struct_size: u32,
    pub scene: u32,
    pub reduce_motion: u32,
    pub knob_count: u32,
    pub knob_values: [f32; MAX_KNOBS],
}

#[repr(C)]
pub struct RuneLabKnobInfo {
    pub struct_size: u32,
    pub min_value: f32,
    pub max_value: f32,
    pub default_value: f32,
    pub kind: u32,
    pub name: [c_char; 32],
    pub label: [c_char; 64],
}

fn text<const N: usize>(s: &str) -> [c_char; N] {
    let mut out = [0; N];
    for (dst, &src) in out.iter_mut().take(N - 1).zip(s.as_bytes()) { *dst = src as c_char; }
    out
}

impl Default for RuneLabConfig {
    fn default() -> Self {
        Self { struct_size: size_of::<Self>() as u32, scene: 0, reduce_motion: 0,
            knob_count: 0, knob_values: [0.0; MAX_KNOBS] }
    }
}

impl RuneLabConfig {
    fn validate(&self) -> Result<(), String> {
        if self.struct_size != size_of::<Self>() as u32 || self.scene as usize >= ALL.len()
            || self.reduce_motion > 1 { return Err("Invalid Lab config size, scene or reduced-motion flag".into()); }
        let knobs = specs(ALL[self.scene as usize]);
        if knobs.len() > MAX_KNOBS { return Err("Lab scene exceeds ABI knob capacity".into()); }
        if self.knob_count == 0 { return Ok(()) }
        if self.knob_count as usize != knobs.len() {
            return Err("knob_count must be zero (defaults) or match scene knob count".into());
        }
        for (i, spec) in knobs.iter().enumerate() {
            let v = self.knob_values[i];
            if !v.is_finite() || v < spec.min || v > spec.max
                || (matches!(spec.kind, Kind::Toggle | Kind::Swatch(_)) && v.fract() != 0.0) {
                return Err(format!("Invalid Lab knob {}", spec.id));
            }
        }
        Ok(())
    }

    fn apply(&self, s: &mut lab::Lab) {
        let scene = ALL[self.scene as usize];
        s.select(scene);
        s.reduced = self.reduce_motion != 0;
        for (i, spec) in specs(scene).iter().enumerate() {
            s.vals[scene.idx()][i] = if self.knob_count == 0 { spec.default } else { self.knob_values[i] };
        }
    }

    fn snapshot(s: &lab::Lab) -> Self {
        let mut out = Self { scene: s.scene.idx() as u32, reduce_motion: s.reduced as u32,
            knob_count: s.vals[s.scene.idx()].len() as u32, ..Self::default() };
        for (i, v) in s.vals[s.scene.idx()].iter().enumerate() { out.knob_values[i] = *v; }
        out
    }
}

/// # Safety
/// A non-null pointer must reference an aligned readable size field, then a full
/// initialized config if the size matches, as specified by the header.
unsafe fn config_read(p: *const RuneLabConfig) -> Result<RuneLabConfig, String> {
    let c = if p.is_null() { RuneLabConfig::default() } else {
        // SAFETY: readable aligned size prefix is the caller's contract.
        if unsafe { p.cast::<u32>().read() } != size_of::<RuneLabConfig>() as u32 {
            return Err("RuneLabConfig.struct_size must match this ABI".into());
        }
        // SAFETY: matching size checked, full initialized storage required by caller.
        unsafe { p.read() }
    };
    c.validate()?;
    Ok(c)
}

pub type RuneLabCallback = Option<unsafe extern "C" fn(u32, *const RuneLabConfig, *mut c_void) -> i32>;

#[unsafe(no_mangle)]
pub extern "C" fn rune_lab_scene_count() -> u32 { ALL.len() as u32 }

/// # Safety
/// Non-null buffer must be writable for capacity bytes. Text is UTF-8.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_lab_scene_name(scene: u32, buffer: *mut c_char, capacity: u32) -> u32 {
    let Some(scene) = ALL.get(scene as usize) else { fail(-1, "Invalid Lab scene"); return 0 };
    let name = scene.id();
    if !buffer.is_null() && capacity > 0 {
        let n = name.len().min(capacity as usize - 1);
        // SAFETY: caller provides storage for capacity bytes; reserve terminator.
        unsafe { std::ptr::copy_nonoverlapping(name.as_ptr(), buffer.cast(), n); buffer.add(n).write(0); }
    }
    name.len() as u32 + 1
}

#[unsafe(no_mangle)]
pub extern "C" fn rune_lab_knob_count(scene: u32) -> u32 {
    let Some(scene) = ALL.get(scene as usize) else { fail(-1, "Invalid Lab scene"); return 0 };
    specs(*scene).len() as u32
}

/// # Safety
/// out must point to a complete writable, naturally aligned knob-info struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_lab_knob_info(scene: u32, index: u32, out: *mut RuneLabKnobInfo) -> i32 {
    if out.is_null() { return fail(-1, "out is null") }
    let Some(scene) = ALL.get(scene as usize) else { return fail(-1, "Invalid Lab scene") };
    let Some(k) = specs(*scene).get(index as usize) else { return fail(-1, "Invalid Lab knob index") };
    let info = RuneLabKnobInfo { struct_size: size_of::<RuneLabKnobInfo>() as u32,
        min_value: k.min, max_value: k.max, default_value: k.default,
        kind: match k.kind { Kind::Slider(_) => 0, Kind::Toggle => 1, Kind::Swatch(_) => 2 },
        name: text(k.id), label: text(k.label) };
    // SAFETY: complete aligned writable storage is required by caller.
    unsafe { out.write(info); }
    0
}

/// # Safety
/// out must point to a complete writable, naturally aligned config struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_lab_default(out: *mut RuneLabConfig) -> i32 {
    if out.is_null() { return fail(-1, "out is null") }
    // SAFETY: complete aligned writable storage is required by caller.
    unsafe { out.write(RuneLabConfig::default()); }
    0
}

fn active() -> Option<rune::AppHandle<lab::Lab>> {
    ACTIVE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

/// # Safety
/// See config_read and header. Non-null config is required, and is copied on entry.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_lab_set_config(config: *const RuneLabConfig) -> i32 {
    if config.is_null() { return fail(-1, "config is null") }
    // SAFETY: config validity is the caller's contract.
    let config = match unsafe { config_read(config) } { Ok(c) => c, Err(e) => return fail(-1, e) };
    let Some(handle) = active() else { return fail(-5, "Lab is not running") };
    handle.update(move |s, _| config.apply(s));
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn rune_lab_quit() -> i32 {
    let Some(handle) = active() else { return fail(-5, "Lab is not running") };
    handle.quit();
    0
}

/// Ensures no stale runtime handle survives normal return or Rust unwinding.
struct ActiveRun;
impl Drop for ActiveRun {
    fn drop(&mut self) { *ACTIVE.lock().unwrap_or_else(|e| e.into_inner()) = None; }
}

/// # Safety
/// Same main-thread, blocking lifetime and callback contract as Settings in header.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_lab_run(config: *const RuneLabConfig, callback: RuneLabCallback, user_data: *mut c_void) -> i32 {
    ERROR.with(|e| e.borrow_mut().clear());
    // SAFETY: config validity is the caller's contract; null selects defaults.
    let config = match unsafe { config_read(config) } { Ok(c) => c, Err(e) => return fail(-1, e) };
    if CONSUMED.swap(true, Ordering::SeqCst) {
        return fail(-2, "Only one native event loop per process (Settings or Lab)");
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut state = lab::Lab::new();
        config.apply(&mut state);
        let last = Rc::new(Cell::new(RuneLabConfig::snapshot(&state)));
        let observed = last.clone();
        let app = lab::app().init(move || state).args(Vec::<String>::new());
        let handle = app.handle();
        *ACTIVE.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle.clone());
        let running = ActiveRun;
        let mut first = true;
        let result = app.on_frame_drawn(move |rt| {
            let snapshot = RuneLabConfig::snapshot(&rt.state);
            if first || snapshot != observed.get() {
                let kind = if first { 1 } else { 2 };
                first = false;
                observed.set(snapshot);
                if let Some(cb) = callback {
                    // SAFETY: caller retains callback/context until run returns;
                    // this snapshot is borrowed only for the duration of the callback.
                    if unsafe { cb(kind, &snapshot, user_data) } != 0 { handle.quit(); }
                }
            }
        }).try_run();
        drop(running);
        if result.is_ok() {
            if let Some(cb) = callback {
                // SAFETY: callback/context retained until run returns.
                unsafe { cb(3, &last.get(), user_data); }
            }
        }
        result
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => fail(-3, e),
        Err(_) => fail(-4, "Rust panic caught while running Lab"),
    }
}

#[cfg(test)]
#[path = "lab_bridge_tests.rs"]
mod tests;
