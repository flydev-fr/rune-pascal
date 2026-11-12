// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
//! C ABI for the shared Settings and Lab demos. See include/rune_pas.h for contracts.
use std::cell::{Cell, RefCell};
use std::ffi::{c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};

mod lab_bridge;
mod ui_bridge;

static CONSUMED: AtomicBool = AtomicBool::new(false);
thread_local! { static ERROR: RefCell<String> = const { RefCell::new(String::new()) }; }

/// C layout: eight 32-bit fields, natural alignment (no packed record).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuneSettings {
    pub struct_size: u32,
    pub appearance: u32,
    pub accent: u32,
    pub launch_at_login: u32,
    pub menu_bar: u32,
    pub sounds: u32,
    pub speed: f32,
    pub reduce_motion: u32,
}

impl From<&settings::Settings> for RuneSettings {
    fn from(s: &settings::Settings) -> Self {
        Self {
            struct_size: size_of::<Self>() as u32,
            appearance: match s.appearance {
                settings::Appearance::Auto => 0,
                settings::Appearance::Light => 1,
                settings::Appearance::Dark => 2,
            },
            accent: s.accent as u32,
            launch_at_login: s.launch_at_login as u32,
            menu_bar: s.menu_bar as u32,
            sounds: s.sounds as u32,
            speed: s.speed,
            reduce_motion: s.reduce_motion as u32,
        }
    }
}

impl RuneSettings {
    fn validated(self) -> Result<settings::Settings, String> {
        if self.struct_size != size_of::<Self>() as u32 {
            return Err("RuneSettings.struct_size must match this ABI".into());
        }
        if self.appearance > 2 || self.accent > 5
            || [self.launch_at_login, self.menu_bar, self.sounds, self.reduce_motion]
                .iter().any(|&v| v > 1)
            || !self.speed.is_finite() || !(0.0..=1.0).contains(&self.speed)
        {
            return Err("Invalid appearance, accent, boolean or speed".into());
        }
        Ok(settings::Settings {
            appearance: match self.appearance {
                1 => settings::Appearance::Light,
                2 => settings::Appearance::Dark,
                _ => settings::Appearance::Auto,
            },
            accent: self.accent as usize,
            launch_at_login: self.launch_at_login != 0,
            menu_bar: self.menu_bar != 0,
            sounds: self.sounds != 0,
            speed: self.speed,
            reduce_motion: self.reduce_motion != 0,
        })
    }
}

pub type RuneSettingsCallback = Option<unsafe extern "C" fn(u32, *const RuneSettings, *mut c_void) -> i32>;

fn fail(code: i32, message: impl Into<String>) -> i32 {
    ERROR.with(|e| *e.borrow_mut() = message.into());
    code
}

#[unsafe(no_mangle)]
pub extern "C" fn rune_pas_abi_version() -> u32 { 1 }

/// # Safety
/// A non-null out must point to writable, naturally aligned RuneSettings storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_settings_default(out: *mut RuneSettings) -> i32 {
    if out.is_null() { return fail(-1, "out is null"); }
    // SAFETY: the caller supplies a writable struct, as documented in the C header.
    unsafe { out.write(RuneSettings::from(&settings::Settings::default())); }
    0
}

/// Returns bytes required including NUL; permits a null buffer for sizing.
/// # Safety
/// A non-null buffer must be writable for capacity bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_pas_last_error(buffer: *mut c_char, capacity: u32) -> u32 {
    ERROR.with(|e| {
        let message = e.borrow();
        if !buffer.is_null() && capacity > 0 {
            let n = message.len().min(capacity as usize - 1);
            // SAFETY: buffer validity is the caller's contract; the bound reserves the NUL.
            unsafe {
                std::ptr::copy_nonoverlapping(message.as_ptr(), buffer.cast(), n);
                buffer.add(n).write(0);
            }
        }
        message.len() as u32 + 1
    })
}

/// # Safety
/// See rune_pas.h: valid config storage, callback/context lifetime, main-thread-only,
/// no exceptions across callbacks, no unload while running. This function blocks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_settings_run(
    config: *const RuneSettings, callback: RuneSettingsCallback, user_data: *mut c_void,
) -> i32 {
    ERROR.with(|e| e.borrow_mut().clear());
    let initial = if config.is_null() {
        RuneSettings::from(&settings::Settings::default())
    } else {
        // SAFETY: caller guarantees an aligned readable struct_size field, then a full
        // struct only if that size matches. Check the prefix before reading the body.
        if unsafe { config.cast::<u32>().read() } != size_of::<RuneSettings>() as u32 {
            return fail(-1, "RuneSettings.struct_size must match this ABI");
        }
        // SAFETY: size checked, full initialized storage is required by the caller.
        unsafe { config.read() }
    };
    let state = match initial.validated() { Ok(s) => s, Err(e) => return fail(-1, e) };
    if CONSUMED.swap(true, Ordering::SeqCst) {
        return fail(-2, "Only one native event loop per process; start a new process to run again");
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let last = std::rc::Rc::new(Cell::new(initial));
        let observed = last.clone();
        let app = rune::App::new(settings::spec()).init(move || state)
            // Never parse the host executable's command-line arguments.
            .args(Vec::<String>::new());
        let handle = app.handle();
        let mut first = true;
        let result = app.on_frame_drawn(move |rt| {
            let snapshot = RuneSettings::from(&rt.state);
            if first || snapshot != observed.get() {
                let kind = if first { 1 } else { 2 };
                first = false;
                observed.set(snapshot);
                if let Some(cb) = callback {
                    // SAFETY: caller retains callback/context until run returns;
                    // snapshot is borrowed for this call only.
                    if unsafe { cb(kind, &snapshot, user_data) } != 0 { handle.quit(); }
                }
            }
        }).try_run();
        if result.is_ok() {
            if let Some(cb) = callback {
                // SAFETY: same lifetime contract as frame callbacks.
                unsafe { cb(3, &last.get(), user_data); }
            }
        }
        result
    }));
    match result {
        Ok(Ok(())) => 0,
        Ok(Err(e)) => fail(-3, e),
        Err(_) => fail(-4, "Rust panic caught while running Rune"),
    }
}

#[cfg(test)]
mod tests;
