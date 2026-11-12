// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
//! Transactional, controlled UI trees for native C/Pascal applications.
mod document;
mod theme;
pub use theme::RuneUiTheme;
mod effects;
mod host;
mod model;
mod view;
pub use effects::RuneUiEffect;
pub use model::RuneUiStyle;

use model::{State, Tree};
use std::ffi::{CStr, CString, c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex, atomic::Ordering};
use std::thread::ThreadId;
use super::{CONSUMED, ERROR, fail};

#[derive(Default)]
struct Builder { staging: Option<Tree>, owner: Option<ThreadId>, committed: Option<Arc<Tree>>, revision: u32 }
static BUILD: Mutex<Builder> = Mutex::new(Builder { staging: None, owner: None, committed: None, revision: 0 });
static ACTIVE: Mutex<Option<rune::AppHandle<State>>> = Mutex::new(None);

fn active() -> Option<rune::AppHandle<State>> { ACTIVE.lock().unwrap_or_else(|e| e.into_inner()).clone() }
fn mutate(f: impl FnOnce(&mut Tree) -> Result<(), String>) -> i32 {
    let mut b = BUILD.lock().unwrap_or_else(|e| e.into_inner());
    if b.owner != Some(std::thread::current().id()) { return fail(-1, "Begin a UI transaction on this thread first"); }
    let Some(t) = b.staging.as_mut() else { return fail(-1, "No UI transaction") };
    match f(t) { Ok(()) => 0, Err(e) => fail(-1, e) }
}

/// # Safety
/// Non-null p must point to a readable, NUL-terminated UTF-8 string.
unsafe fn string(p: *const c_char) -> Result<String, String> {
    if p.is_null() { return Ok(String::new()) }
    // SAFETY: valid terminated string is the caller's contract.
    let s = unsafe { CStr::from_ptr(p) }.to_str().map_err(|_| "Text must be UTF-8")?;
    if s.len() > 65536 { return Err("Text exceeds 64 KiB".into()) }
    Ok(s.to_owned())
}

#[unsafe(no_mangle)]
pub extern "C" fn rune_ui_begin() -> i32 {
    let mut b = BUILD.lock().unwrap_or_else(|e| e.into_inner());
    if b.owner.is_some_and(|id| id != std::thread::current().id()) { return fail(-1, "UI transaction belongs to another thread") }
    b.staging = Some(Tree::default()); b.owner = Some(std::thread::current().id()); 0
}

#[unsafe(no_mangle)]
pub extern "C" fn rune_ui_abort() -> i32 {
    let mut b = BUILD.lock().unwrap_or_else(|e| e.into_inner());
    if b.owner != Some(std::thread::current().id()) { return fail(-1, "No UI transaction on this thread") }
    b.staging = None; b.owner = None; 0
}

/// # Safety
/// text must be null or a valid NUL-terminated UTF-8 string. It is copied.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_add(parent: u32, id: u32, kind: u32, text: *const c_char) -> i32 {
    // SAFETY: valid terminated string required by caller.
    let value = match unsafe { string(text) } { Ok(s) => s, Err(e) => return fail(-1, e) };
    mutate(move |t| t.add(parent, id, kind, value))
}

/// # Safety
/// out must point to a complete writable aligned RuneUiStyle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_style_default(out: *mut RuneUiStyle) -> i32 {
    if out.is_null() { return fail(-1, "out is null") }
    // SAFETY: complete writable storage required by caller.
    unsafe { out.write(RuneUiStyle::default()); } 0
}

/// # Safety
/// style must reference a readable aligned size prefix, and the full struct when matching.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_set_style(id: u32, style: *const RuneUiStyle) -> i32 {
    if style.is_null() { return fail(-1, "style is null") }
    // SAFETY: caller guarantees readable size prefix.
    if unsafe { style.cast::<u32>().read() } != size_of::<RuneUiStyle>() as u32 { return fail(-1, "Invalid style size") }
    // SAFETY: full readable struct required after matching size.
    let style = unsafe { style.read() };
    if !style.valid() { return fail(-1, "Invalid UI style") }
    mutate(|t| {
        let n=t.nodes.get(&id).ok_or("Unknown UI node")?;
        if style.flags&1024!=0 && n.kind!=4 {return Err("Default action must be a button".into())}
        if style.flags&512!=0 {
            if n.kind!=1 || !t.nodes.get(&t.root).is_some_and(|root|root.children.contains(&id)) {
                return Err("Modal must be a column directly under the root".into())
            }
            if t.nodes.values().any(|n|n.id!=id && n.style.flags&512!=0) {return Err("Only one modal per tree".into())}
        }
        t.nodes.get_mut(&id).unwrap().style = style; Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn rune_ui_set_checked(id: u32, checked: u32) -> i32 {
    if checked > 1 { return fail(-1, "checked must be zero or one") }
    mutate(|t| { let n = t.nodes.get_mut(&id).ok_or("Unknown UI node")?;
        if n.kind != 6 { return Err("Node is not a checkbox".into()) }
        n.checked = checked != 0; Ok(()) })
}

#[unsafe(no_mangle)]
pub extern "C" fn rune_ui_commit() -> i32 {
    let tree = {
        let mut b = BUILD.lock().unwrap_or_else(|e| e.into_inner());
        if b.owner != Some(std::thread::current().id()) { return fail(-1, "No UI transaction on this thread") }
        if !b.staging.as_ref().is_some_and(Tree::valid) { return fail(-1, "UI tree has no root") }
        let Some(revision) = b.revision.checked_add(1) else { return fail(-1, "UI revision exhausted") };
        let mut tree = b.staging.take().unwrap(); tree.revision = revision;
        let tree = Arc::new(tree); b.owner = None; b.revision = revision; b.committed = Some(tree.clone()); tree
    };
    if let Some(h) = active() { h.update(move |s, _| s.install(tree)); }
    0
}

/// # Safety
/// out must point to a complete writable aligned RuneUiEffect.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_effect_default(kind: u32, out: *mut RuneUiEffect) -> i32 {
    if out.is_null() || kind > 8 { return fail(-1, "Invalid effect kind or output") }
    // SAFETY: writable full output storage required by caller.
    unsafe { out.write(RuneUiEffect::preset(kind)); } 0
}

/// # Safety
/// effect must reference a readable aligned size prefix, then a full matching struct.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_set_effect(id: u32, effect: *const RuneUiEffect) -> i32 {
    if effect.is_null() { return fail(-1, "effect is null") }
    // SAFETY: caller supplies readable size prefix.
    if unsafe { effect.cast::<u32>().read() } != size_of::<RuneUiEffect>() as u32 { return fail(-1, "Invalid effect size") }
    // SAFETY: full initialized struct required after matching size.
    let effect = unsafe { effect.read() };
    if !effect.valid() { return fail(-1, "Invalid shader effect parameters") }
    mutate(|tree| {
        let n = tree.nodes.get_mut(&id).ok_or("Unknown UI node")?;
        if n.kind != 4 { return Err("Shader effects currently attach to buttons only".into()) }
        n.effect = (effect.kind != 0).then_some(effect);
        Ok(())
    })
}

pub type RuneUiCallback = Option<unsafe extern "C" fn(u32, u32, u32, *const c_char, *mut c_void) -> i32>;
fn notify(cb: RuneUiCallback, kind: u32, id: u32, value: u32, text: &str, user: *mut c_void) -> bool {
    let Some(cb) = cb else { return false };
    let text = CString::new(text.replace('\0', "�")).unwrap();
    // SAFETY: callback/context retained for run lifetime; temporary text borrowed only during call.
    unsafe { cb(kind, id, value, text.as_ptr(), user) != 0 }
}

struct Running;
impl Drop for Running { fn drop(&mut self) { *ACTIVE.lock().unwrap_or_else(|e| e.into_inner()) = None; } }

#[unsafe(no_mangle)]
pub extern "C" fn rune_ui_quit() -> i32 {
    let Some(h) = active() else { return fail(-5, "UI is not running") }; h.quit(); 0
}

/// # Safety
/// title is readable NUL-terminated UTF-8; callbacks follow the header contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_run(title: *const c_char, width: u32, height: u32, callback: RuneUiCallback, user: *mut c_void) -> i32 {
    ERROR.with(|e| e.borrow_mut().clear());
    // SAFETY: caller provides terminated title.
    let title = match unsafe { string(title) } { Ok(s) => s, Err(e) => return fail(-1, e) };
    if title.len() > 1024 || !(240..=8192).contains(&width) || !(240..=8192).contains(&height) { return fail(-1, "Invalid window title or size") }
    let tree = { let b = BUILD.lock().unwrap_or_else(|e| e.into_inner());
        if b.staging.is_some() { return fail(-1, "Commit or abort the pending UI transaction first") }
        b.committed.clone() };
    let Some(tree) = tree else { return fail(-1, "Commit a UI tree before run") };
    if CONSUMED.swap(true, Ordering::SeqCst) { return fail(-2, "Only one native event loop per process") }
    let result = catch_unwind(AssertUnwindSafe(|| {
        // Rune names apps with &'static str. One bounded title allocation is intentionally
        // retained for the process lifetime, matching the single permitted native run.
        let title = Box::leak(title.into_boxed_str());
        let spec = rune::AppSpec::new(title, (width as f32, height as f32),
            || unreachable!("init overridden"), view::view);
        let app = rune::App::new(spec).init(move || State::new(tree)).args(Vec::<String>::new()).compositor(effects::Effects::default());
        let handle = app.handle();
        *ACTIVE.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle.clone());
        let running = Running;
        let mut first = true; let mut revision = 0;
        let result = app.on_frame_drawn(move |rt| {
            let current = rt.state.tree.revision;
            let events = std::mem::take(&mut *rt.state.events.borrow_mut());
            let quit = if first { first = false; revision = current; notify(callback, 1, 0, current, "", user) }
                else if revision != current { revision = current; notify(callback, 14, 0, current, "", user) } else { false };
            if quit { handle.quit(); return }
            for e in events { if notify(callback, e.kind, e.id, e.value, &e.text, user) { handle.quit(); break } }
        }).try_run();
        drop(running);
        if result.is_ok() { notify(callback, 3, 0, 0, "", user); }
        result
    }));
    match result { Ok(Ok(())) => 0, Ok(Err(e)) => fail(-3, e), Err(_) => fail(-4, "Rust panic caught while running UI") }
}

#[cfg(test)]
mod tests;

/// Set a native document node from the formatted output of an external Markdown parser.
/// # Safety
/// All strings are valid NUL-terminated UTF-8, copied before return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_set_document(id:u32, markup:*const c_char, base:*const c_char, query:*const c_char)->i32{
    if markup.is_null(){return fail(-1,"Document is null")}
    // SAFETY: valid NUL-terminated strings are required by the caller.
    let markup=match unsafe{CStr::from_ptr(markup)}.to_str(){Ok(s) if s.len()<=2*1024*1024=>s, _=>return fail(-1,"Invalid document UTF-8/size")};
    let base=match unsafe{string(base)}{Ok(s)=>s,Err(e)=>return fail(-1,e)};
    let query=match unsafe{string(query)}{Ok(s)=>s,Err(e)=>return fail(-1,e)};
    let doc=match document::Document::new(markup,&base,&query){Ok(d)=>Arc::new(d),Err(e)=>return fail(-1,e)};
    mutate(|tree|{let n=tree.nodes.get_mut(&id).ok_or("Unknown document node")?;if n.kind!=7{return Err("Node is not a document".into())}n.document=Some(doc);Ok(())})
}

/// # Safety
/// out is a complete writable aligned RuneUiTheme.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_theme_default(dark:u32,out:*mut RuneUiTheme)->i32 {
 if out.is_null()||dark>1{return fail(-1,"Invalid theme output or mode")}
 unsafe{out.write(RuneUiTheme::preset(dark!=0));} 0
}
/// # Safety
/// theme has a readable size prefix and a complete matching record.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_set_theme(theme:*const RuneUiTheme)->i32 {
 if theme.is_null(){return fail(-1,"Null theme")}
 if unsafe{theme.cast::<u32>().read()}!=size_of::<RuneUiTheme>() as u32{return fail(-1,"Invalid theme size")}
 let theme=unsafe{theme.read()};if !theme.valid(){return fail(-1,"Invalid theme values")}
 mutate(|t|{t.theme=Some(theme);Ok(())})
}
/// # Safety
/// hint is null or valid NUL-terminated UTF-8, copied before return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_set_hint(id:u32,hint:*const c_char)->i32 {
 let hint=match unsafe{string(hint)}{Ok(s) if s.len()<=1024=>s,_=>return fail(-1,"Invalid hint")};
 mutate(|t|{t.nodes.get_mut(&id).ok_or("Unknown node")?.hint=hint;Ok(())})
}

/// Set the controlled zero-based combobox selection.
#[unsafe(no_mangle)]
pub extern "C" fn rune_ui_set_choice(id: u32, index: u32) -> i32 {
    mutate(|t| {
        let n = t.nodes.get_mut(&id).ok_or("Unknown UI node")?;
        if n.kind != 9 || index as usize >= n.text.split('\n').count() {
            return Err("Invalid combobox or option index".into());
        }
        n.choice = index; Ok(())
    })
}
