// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
//! Typed button effects, shared with Rune Lab's rune-fx shader implementation.
use std::collections::{HashMap, HashSet};
use rune::compose::{Compositor, DeviceKey, Frame};
use rune::render::{EffectPhase, EffectSite, KIND_EFFECT};
use rune_fx::*;
use super::model::State;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuneUiEffect {
    pub struct_size: u32,
    pub kind: u32,
    pub color_rgba: u32,
    pub flags: u32,
    pub intensity: f32,
    pub speed: f32,
    pub size: f32,
    pub parameter: f32,
}
impl RuneUiEffect {
    pub fn preset(kind: u32) -> Self {
        Self { struct_size: size_of::<Self>() as u32, kind, color_rgba: 0x67bcffff,
            flags: 0, intensity: 1.0, speed: 1.0,
            size: match kind { 3 => 34.0, 4 => 28.0, 5 | 6 => 8.0, _ => 14.0 },
            parameter: match kind { 5 => 0.6, 6 => 0.15, 8 => 0.2, _ => 0.0 } }
    }
    pub fn valid(&self) -> bool {
        self.struct_size == size_of::<Self>() as u32 && self.kind <= 8 && self.flags & !3 == 0
            && self.intensity.is_finite() && (0.0..=3.0).contains(&self.intensity)
            && self.speed.is_finite() && (0.0..=4.0).contains(&self.speed)
            && self.size.is_finite() && (0.0..=128.0).contains(&self.size)
            && self.parameter.is_finite() && (0.0..=1.0).contains(&self.parameter)
    }
    pub fn layer(&self, shape: Shape, time: f32) -> FxLayer {
        let color = rgb(self.color_rgba >> 8);
        let fx = match self.kind {
            1 => Fx::glow(GlowParams { style: GlowStyle::Neon, color,
                intensity: self.intensity, radius: self.size, breathe: self.parameter, ..Default::default() }),
            2 => Fx::electric(ElectricParams { color, intensity: self.intensity,
                amplitude: self.size * 0.35, ..ElectricParams::charged() }),
            3 => Fx::fire(FireParams { height: self.size, intensity: self.intensity,
                embers: 12, ..Default::default() }),
            4 => Fx::shimmer(ShimmerParams { intensity: self.intensity * 0.3,
                band: self.size, ..Default::default() }),
            5 => Fx::fill(LiquidMetalParams { light: color, flow: self.parameter,
                bevel: self.size, ..LiquidMetalParams::chrome() }.anchor(FillAnchor::Element)),
            6 => Fx::fill(PlasticParams { color, roughness: self.parameter,
                bevel: self.size, ..PlasticParams::glossy() }),
            7 => Fx::pulse(PulseParams { style: PulseStyle::Breathe, color,
                intensity: self.intensity, reach: self.size, ..Default::default() }),
            8 => Fx::lens(LensParams { depth: self.size, frost: self.parameter,
                bevel: 10.0, ..Default::default() }),
            _ => Fx::glow(GlowParams { intensity: 0.0, ..Default::default() }),
        };
        let alpha = (self.color_rgba & 255) as f32 / 255.0;
        let opacity = if self.intensity == 0.0 { 0.0 } else if self.kind >= 5 && self.kind != 7 { self.intensity.min(1.0) } else { 1.0 };
        fx.on(shape).at(time).opacity(alpha * opacity).reduced(self.flags & 2 != 0)
    }
}

#[derive(Default)]
pub struct Effects {
    gpu: Option<(DeviceKey, FxRenderer)>,
    configs: HashMap<u32, RuneUiEffect>,
    phases: HashMap<u32, Phase>,
    live: bool,
}
impl Compositor<State> for Effects {
    fn animating(&self) -> bool { self.live }
    fn render(&mut self, f: &mut Frame<'_, State>) {
        let t = f.rt.t as f32;
        let mut at = HashMap::new();
        let mut layers = Vec::new();
        let mut slots = Vec::new();
        let mut seen = HashSet::new();
        self.live = false;
        for (index, i) in f.rt.instances.iter().enumerate() {
            if i.params[1] != KIND_EFFECT { continue }
            let id = f.rt.marks.get(index).copied().unwrap_or(0);
            let config = f.rt.state.tree.nodes.get(&id).and_then(|n| n.effect)
                .or_else(|| self.configs.get(&id).copied());
            let Some(config) = config else { continue };
            seen.insert(id);
            self.configs.insert(id, config);
            let scale = i.params[3];
            let [x, y, w, h] = i.rect.map(|v| v / scale);
            let radius = i.radii.iter().fold(0.0f32, |a, v| a.max(v.abs())) / scale;
            let phase = self.phases.entry(id).or_insert_with(|| Phase::new(config.speed));
            phase.set_speed(t, config.speed);
            let mut layer = config.layer(Shape::rounded(x, y, w, h, radius), phase.at(t)).seed(id);
            if config.flags & 1 != 0 {
                let inside = f.rt.pointer().is_some_and(|(px, py)| px >= x && px < x+w && py >= y && py < y+h
                    && px*scale >= i.clip[0] && py*scale >= i.clip[1] && px*scale < i.clip[2] && py*scale < i.clip[3]);
                if !inside { layer.opacity = 0.0; }
            }
            self.live |= config.speed > 0.0 && layer.is_animating();
            at.insert(index as u32, layers.len());
            layers.push(layer);
            slots.push(FxSlot::default().scale(scale).clip(i.clip, i.fade));
        }
        self.configs.retain(|id, _| seen.contains(id));
        self.phases.retain(|id, _| seen.contains(id));
        if layers.is_empty() { f.draw_ui(); return }
        let key = f.device_key();
        if self.gpu.as_ref().is_none_or(|(old, _)| *old != key) {
            self.gpu = Some((key, FxRenderer::new(f.device, f.format, FxOptions { dither: true })));
        }
        let fx = &mut self.gpu.as_mut().unwrap().1;
        fx.set_srgb_blend(f.srgb_blend());
        fx.add_format(f.device, wgpu::TextureFormat::Rgba16Float);
        fx.prepare_with_slots(f.device, f.queue, &FxFrame {
            viewport: [f.size.0, f.size.1], scale: 1.0, layers: &layers,
            backdrop: None, sources: &[],
        }, &slots);
        let clear = f.rt.clear_color();
        f.ui.prepare(f.device, f.queue, &mut f.rt.text.atlas, &f.rt.instances, f.size);
        f.ui.draw_with_backdrop(f.encoder, f.target, clear, &mut |phase, site: &EffectSite<'_>| {
            let Some(&index) = at.get(&site.index) else { return };
            match phase {
                EffectPhase::Backdrop(encoder) => {
                    if let Some(backdrop) = site.backdrop { fx.blur_backdrop(f.device, f.queue, encoder, backdrop, index); }
                },
                EffectPhase::Draw(pass) => fx.draw_layer_in(pass, site.format, site.origin, index),
            }
        });
    }
}
