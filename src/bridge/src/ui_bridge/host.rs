// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
//! Win32 borrowed-HWND surface. VCL owns the HWND, message pump and lifetime.
use super::{BUILD, RuneUiCallback, effects::Effects, model::State, notify, view};
use crate::fail;
use rune::compose::{Compositor, Frame};
use rune::render::{Gpu, Renderer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::num::NonZeroIsize;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::time::Instant;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RuneHostFrame {
    pub struct_size: u32,
    pub flags: u32,
    pub update_ms: f64,
    pub encode_ms: f64,
    pub present_ms: f64,
    pub total_ms: f64,
}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RuneBenchResult {
    pub struct_size: u32,
    pub frames: u32,
    pub encode_median_ms: f64,
    pub encode_p95_ms: f64,
    pub encode_mean_ms: f64,
    pub elapsed_ms: f64,
    pub cpu_ms: f64,
}

// Handle lookup is thread-local: a foreign/stale handle is rejected, never dereferenced.
thread_local! {
    static HOSTS: RefCell<HashMap<u32, Host>> = RefCell::new(HashMap::new());
}
static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
fn next_id() -> Result<u32,String> {
    NEXT.fetch_update(std::sync::atomic::Ordering::Relaxed,std::sync::atomic::Ordering::Relaxed,|n|n.checked_add(1))
        .map_err(|_| "Host IDs exhausted".into())
}
struct Host {
    // Surface must be dropped before the caller destroys the borrowed HWND.
    surface: wgpu::Surface<'static>,
    gpu: Gpu,
    renderer: Renderer,
    effects: Effects,
    rt: rune::Runtime<State>,
    config: wgpu::SurfaceConfiguration,
    scale: f32,
    callback: RuneUiCallback,
    user: *mut c_void,
    first: bool,
    revision: u32,
    dirty: bool,
    last: Instant,
}
fn current_tree() -> Result<std::sync::Arc<super::model::Tree>, String> {
    let b = BUILD.lock().unwrap_or_else(|e| e.into_inner());
    if b.staging.is_some() { return Err("Commit or abort the pending tree first".into()) }
    b.committed.clone().ok_or("Commit a UI tree first".into())
}
fn protect(f: impl FnOnce() -> Result<(), String>) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => 0, Ok(Err(e)) => fail(-1, e), Err(_) => fail(-4, "Rust panic caught in embedded host"),
    }
}
fn with_host<T>(id: u32, f: impl FnOnce(&mut Host) -> Result<T, String>) -> Result<T, String> {
    HOSTS.with(|hosts| {
        let mut hosts = hosts.try_borrow_mut().map_err(|_| "Reentrant host operation")?;
        f(hosts.get_mut(&id).ok_or("Invalid/stale host or wrong thread")?)
    })
}
fn valid_size(w: u32, h: u32, scale: f32) -> bool {
    (1..=16384).contains(&w) && (1..=16384).contains(&h) && scale.is_finite() && (0.5..=8.0).contains(&scale)
}

/// # Safety
/// hwnd is a live same-thread Win32 HWND, retained until host_destroy returns.
/// out is complete writable u32 storage. Callback contracts match RuneUiCallback.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_host_create(hwnd: usize, width: u32, height: u32, scale: f32,
    callback: RuneUiCallback, user: *mut c_void, out: *mut u32) -> i32 {
    if out.is_null() || hwnd == 0 || !valid_size(width, height, scale) { return fail(-1, "Invalid HWND, output or dimensions") }
    // SAFETY: caller supplies output storage; failed creation always returns zero handle.
    unsafe { out.write(0); }
    protect(|| {
        let tree = current_tree()?;
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle_from_env();
        desc.backends = wgpu::Backends::DX12;
        let instance = wgpu::Instance::new(desc);
        let window = Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).ok_or("Invalid HWND")?);
        // SAFETY: borrowed HWND remains live until explicit host_destroy; caller owns it.
        let surface = unsafe { instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(RawDisplayHandle::Windows(WindowsDisplayHandle::new())),
            raw_window_handle: RawWindowHandle::Win32(window),
        }) }.map_err(|e| e.to_string())?;
        let gpu = Gpu::try_new(instance, Some(&surface))?;
        eprintln!("RUNE_HOST adapter={:?} physical={}x{} scale={}",gpu.adapter.get_info(),width,height,scale);
        let mut config = surface.get_default_config(&gpu.adapter, width, height).ok_or("No surface configuration")?;
        // Same gamma-space blend convention as Rune's native shell.
        let target = config.format.remove_srgb_suffix();
        config.view_formats = vec![target];
        config.present_mode = wgpu::PresentMode::Fifo;
        config.usage |= wgpu::TextureUsages::COPY_SRC;
        surface.configure(&gpu.device, &config);
        let mut renderer = Renderer::new(&gpu.device, target);
        renderer.set_srgb_blend(true);
        let mut rt = rune::Runtime::new(State::new(tree), view::view,
            (width as f32 / scale, height as f32 / scale), rune::Env::new(true, scale));
        rt.set_srgb_blend(true); rt.track_marks = true;
        let id = next_id()?;
        HOSTS.with(|h| h.borrow_mut().insert(id, Host { surface, gpu, renderer, effects: Effects::default(), rt,
            config, scale, callback, user, first: true, revision: 0, dirty: true, last: Instant::now() }));
        // SAFETY: complete output storage was checked by caller contract.
        unsafe { out.write(id); }
        Ok(())
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn rune_host_destroy(id: u32) -> i32 {
    protect(|| HOSTS.with(|hosts| hosts.try_borrow_mut().map_err(|_| "Host is busy")?
        .remove(&id).map(|_| ()).ok_or("Invalid/stale host or wrong thread".into())))
}
#[unsafe(no_mangle)]
pub extern "C" fn rune_host_commit(id: u32) -> i32 {
    protect(|| { let tree = current_tree()?; with_host(id, |h| { h.rt.state.install(tree); h.rt.invalidate(); h.dirty = true; Ok(()) }) })
}
#[unsafe(no_mangle)]
pub extern "C" fn rune_host_resize(id: u32, width: u32, height: u32, scale: f32) -> i32 {
    if !valid_size(width, height, scale) { return fail(-1, "Invalid dimensions") }
    protect(|| with_host(id, |h| {
        h.config.width = width; h.config.height = height; h.scale = scale;
        h.surface.configure(&h.gpu.device, &h.config);
        h.rt.set_viewport((width as f32 / scale, height as f32 / scale), scale); h.dirty = true; Ok(())
    }))
}

// Keep keyboard decoding shared with the headless clipboard regression tests.
fn key_input(rt:&mut rune::Runtime<State>,value:u32,flags:u32)->Result<(),String>{
    let key=match value {
        65|67 if flags&2!=0=>rune::Key::Command(if value==65 {'a'}else{'c'}),
        8=>rune::Key::Backspace,46=>rune::Key::Delete,36=>rune::Key::Home,35=>rune::Key::End,
        33=>rune::Key::PageUp,34=>rune::Key::PageDown,65=>rune::Key::Text("a".into()),
        67=>rune::Key::Text("c".into()),9=>rune::Key::Tab,13=>rune::Key::Enter,
        32=>rune::Key::Text(" ".into()),27=>rune::Key::Escape,37=>rune::Key::Left,
        38=>rune::Key::Up,39=>rune::Key::Right,40=>rune::Key::Down,
        _=>return Err("Unsupported prototype key".into())
    };
    let mods=rune::Mods{shift:flags&1!=0,ctrl:flags&2!=0,cmd:flags&2!=0,..Default::default()};
    rt.key(rune::KeyEvent{key,mods});Ok(())
}
fn take_copy_event(rt:&mut rune::Runtime<State>)->Option<super::model::Event>{
    let copy=rt.copy_request.take()?;
    Some(super::model::Event{kind:17,id:0,value:0,text:copy.content().text().to_owned()})
}

/// Mouse coordinates are physical client pixels. kind=1 move,2 down,3 up,4 leave,
/// 5 key(VK_TAB/RETURN/SPACE/ESC/arrows),6 focus lost. flags bit0 shift,bit1 ctrl.
#[unsafe(no_mangle)]
pub extern "C" fn rune_host_input(id: u32, kind: u32, x: f32, y: f32, value: u32, flags: u32) -> i32 {
    if !x.is_finite() || !y.is_finite() { return fail(-1, "Invalid pointer coordinates") }
    protect(|| with_host(id, |h| {
        let (x,y) = (x/h.scale, y/h.scale);
        match kind {
            1 => h.rt.pointer_move(x,y),
            2 => { h.rt.pointer_move(x,y); h.rt.pointer_down(x,y); },
            3 => h.rt.pointer_up(x,y),
            4 | 6 => h.rt.pointer_leave(),
            8 => {if let Some(c)=char::from_u32(value){h.rt.key(rune::KeyEvent{key:rune::Key::Text(c.to_string()),mods:Default::default()});}},
            7 => h.rt.wheel(x,y,(value as i32) as f32),
            5 => {
                key_input(&mut h.rt,value,flags)?;
            },
            _ => return Err("Unsupported prototype input".into()),
        }
        h.dirty=true; Ok(())
    }))
}

/// # Safety
/// out points to complete writable RuneHostFrame. All calls on creating thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_host_frame(id: u32, out: *mut RuneHostFrame) -> i32 {
    if out.is_null() { return fail(-1, "out is null") }
    protect(|| {
        let (metrics, callback, user, events) = with_host(id, |h| {
            let start = Instant::now();
            let dt = h.last.elapsed().as_secs_f64(); h.last = Instant::now();
            let moving = h.rt.frame(dt);
            if let Some(copy)=take_copy_event(&mut h.rt){h.rt.state.events.borrow_mut().push(copy);}
            let update_ms = start.elapsed().as_secs_f64()*1000.0;
            let acquired = Instant::now();
            let texture = match h.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t)|wgpu::CurrentSurfaceTexture::Suboptimal(t)=>t,
                wgpu::CurrentSurfaceTexture::Outdated|wgpu::CurrentSurfaceTexture::Lost=>{
                    h.surface.configure(&h.gpu.device,&h.config); h.dirty=true;
                    return Err("Surface reconfigured; retry next frame".into())
                },
                _=>return Err("No drawable (occluded or timeout)".into()),
            };
            let acquire_ms=acquired.elapsed().as_secs_f64()*1000.0;
            let format=h.config.format.remove_srgb_suffix();
            let target=texture.texture.create_view(&wgpu::TextureViewDescriptor { format:Some(format),..Default::default() });
            let mut encoder=h.gpu.device.create_command_encoder(&Default::default());
            let encode=Instant::now();
            h.effects.render(&mut Frame { device:&h.gpu.device,queue:&h.gpu.queue,encoder:&mut encoder,target:&target,
                format,size:(h.config.width,h.config.height),dt,update_ms,rt:&mut h.rt,ui:&mut h.renderer });
            let encode_ms=encode.elapsed().as_secs_f64()*1000.0;
            let present=Instant::now();
            h.renderer.submit(&h.gpu.queue,encoder); h.gpu.queue.present(texture);
            let present_ms=acquire_ms+present.elapsed().as_secs_f64()*1000.0;
            let live=moving||h.effects.animating(); h.dirty=live;
            let mut events=std::mem::take(&mut *h.rt.state.events.borrow_mut());
            let revision=h.rt.state.tree.revision;
            if h.first { h.first=false; h.revision=revision; events.insert(0,super::model::Event {kind:1,id:0,value:revision,text:String::new()}); }
            else if h.revision!=revision { h.revision=revision; events.insert(0,super::model::Event {kind:14,id:0,value:revision,text:String::new()}); }
            Ok((RuneHostFrame {struct_size:size_of::<RuneHostFrame>() as u32,flags:live as u32,update_ms,encode_ms,present_ms,
                total_ms:start.elapsed().as_secs_f64()*1000.0},h.callback,h.user,events))
        })?;
        // SAFETY: complete output storage required by caller. No registry borrow during callbacks.
        unsafe { out.write(metrics); }
        for e in events { if notify(callback,e.kind,e.id,e.value,&e.text,user) { break } }
        Ok(())
    })
}

// Benchmark wrapper: measure exactly the same compositor CPU encoding scope as host_frame.
#[derive(Default)]
struct Samples { n:u32, warm:u32, wanted:u32, values:Vec<f64>, start:Option<Instant>, end:Option<Instant>, result:Option<RuneBenchResult> }
struct Measured { effects:Effects, samples:Rc<RefCell<Samples>> }
impl Compositor<State> for Measured {
    fn animating(&self)->bool { true } // controlled benchmark deliberately renders continuously
    fn render(&mut self,f:&mut Frame<'_,State>) {
        let start=Instant::now(); self.effects.render(f);
        let ms=start.elapsed().as_secs_f64()*1000.0;
        let mut s=self.samples.borrow_mut(); s.n+=1;
        if s.n==s.warm+1 {s.start=Some(Instant::now());}
        if s.n>s.warm {s.values.push(ms);}
    }
}
/// # Safety
/// out points to a complete writable RuneBenchResult. Native event-loop rules apply.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_ui_benchmark(frames:u32,out:*mut RuneBenchResult)->i32 {
    if out.is_null()||!(30..=3600).contains(&frames){return fail(-1,"Invalid benchmark output/frame count")}
    protect(||{
        let tree=current_tree()?;
        if crate::CONSUMED.swap(true,std::sync::atomic::Ordering::SeqCst){return Err("One native event loop per process".into())}
        let samples=Rc::new(RefCell::new(Samples{warm:60,wanted:frames,..Default::default()}));
        let measured=Measured{effects:Effects::default(),samples:samples.clone()};
        let app=rune::App::new(rune::AppSpec::new("Rune standalone benchmark",(700.0,260.0),||unreachable!(),view::view))
            .init(move||State::new(tree)).args(Vec::<String>::new()).compositor(measured);
        let handle=app.handle();let observed=samples.clone();
        app.on_frame_drawn(move|_|{let s=observed.borrow();if s.n>=s.warm+s.wanted{drop(s);observed.borrow_mut().end=Some(Instant::now());handle.quit();}}).try_run()?;
        let mut s=samples.borrow_mut();
        let elapsed=s.start.zip(s.end).map(|(a,b)|(b-a).as_secs_f64()*1000.0).unwrap_or(0.0);
        let n=s.values.len(); if n==0{return Err("No measured frames".into())}
        s.values.sort_by(f64::total_cmp);
        let result=RuneBenchResult{struct_size:size_of::<RuneBenchResult>() as u32,frames:n as u32,
            encode_median_ms:s.values[n/2],encode_p95_ms:s.values[(n*95/100).min(n-1)],
            encode_mean_ms:s.values.iter().sum::<f64>()/n as f64,elapsed_ms:elapsed,cpu_ms:0.0};
        // SAFETY: caller guarantees complete writable output storage.
        unsafe{out.write(result)};s.result=Some(result);Ok(())
    })
}

#[cfg(test)]
#[path = "host_tests.rs"]
mod tests;

/// Choose the actual native font families behind both text slots.
/// # Safety
/// Strings are readable NUL-terminated UTF-8, copied before return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rune_host_set_fonts(id:u32,sans:*const std::ffi::c_char,mono:*const std::ffi::c_char)->i32{
    // SAFETY: caller supplies complete readable strings.
    let sans=match unsafe{super::string(sans)}{Ok(s)=>s,Err(e)=>return fail(-1,e)};
    let mono=match unsafe{super::string(mono)}{Ok(s)=>s,Err(e)=>return fail(-1,e)};
    if sans.is_empty()||mono.is_empty()||sans.len()>256||mono.len()>256{return fail(-1,"Invalid font families")}
    protect(||with_host(id,|h|{h.rt.set_font_families(&sans,&mono);h.rt.invalidate();h.dirty=true;Ok(())}))
}
