// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
use super::model::*;
use rune_kit::{ScrollBar,ScrollBarStyle,Combobox,ComboboxOption,Kit};
use rune::{Color, Cx, El, ScrollSpec, Spring, TextStyle, Transition, col, row, text, text_input};

pub(super) fn color(c: u32) -> Color {
    Color { r: rune::color::srgb_decode(((c >> 24) & 255) as f32 / 255.0),
        g: rune::color::srgb_decode(((c >> 16) & 255) as f32 / 255.0),
        b: rune::color::srgb_decode(((c >> 8) & 255) as f32 / 255.0), a: (c & 255) as f32 / 255.0 }
}

fn node(s: &State, id: u32, layers: &mut Vec<El>) -> El {
    let n = &s.tree.nodes[&id];
    let st = n.style;
    let theme=s.tree.theme.unwrap_or_else(||super::RuneUiTheme::preset(color(st.foreground_rgba).luminance()>0.4));
    let font = TextStyle::new(st.font_size, st.font_weight as f32);
    let ink = color(st.foreground_rgba);
    let mut e = match n.kind {
        1 => col(),
        2 => row().center_y(),
        3 => text(n.text.clone()).font(font).color(ink),
        4 => row().center().role("button").label(n.text.clone()).focusable(true)
            .on_press(move |s: &mut State| s.emit(10, id, 0, String::new()))
,
        5 => text_input(&s.edits[&id], font).single_line().color(ink)
            .caret(ink, color(0x4389f566))
            .on_edit(move |s: &mut State| s.edits.get_mut(&id).unwrap())
            .on_input_change(move |s: &mut State, _| s.emit(11, id, 0, s.edits[&id].text.clone()))
            .on_submit(move |s: &mut State, _| s.emit(12, id, 0, s.edits[&id].text.clone())),
        6 => {
            let checked = n.checked;
            let mark = row().size(22.0, 22.0).radius(6.0).center()
                .fill(color(if checked { 0x4389f5ff } else { 0x424752ff }))
                .child(text(if checked { "✓" } else { "" }).font(font).color(Color::WHITE));
            row().gap(12.0).center_y().role("checkbox").label(n.text.clone()).checked(checked).focusable(true)
                .on_press(move |s: &mut State| s.emit(13, id, (!checked) as u32, String::new()))
                .child(mark).child(text(n.text.clone()).font(font).color(ink).wrap().grow(1.0))
        },
        8 => {
            let (title,detail)=n.text.split_once('\n').unwrap_or((&n.text,""));
            let e=col().gap(3.0).role("button").label(n.text.clone()).focusable(true)
                .on_press(move|s:&mut State|s.emit(10,id,0,String::new()))
                .hover(ink.alpha(0.055)).cursor(rune::Cursor::Pointer)
                .child(text(title.to_owned()).font(font).color(ink).w_full().truncate());
            if detail.is_empty(){e}else{e.child(text(detail.to_owned()).font(TextStyle::new(10.0,400.0)).color(ink.alpha(0.55)))}
        },
        9 => {
            let options: Vec<_> = n.text.split('\n').enumerate()
                .map(|(i, label)| ComboboxOption::new(i.to_string(), label)).collect();
            let mut kit = Kit::neutral(ink.luminance() > 0.4);
            kit.c.text_base = ink; kit.c.text_muted = color(theme.muted);
            kit.c.icon_base = ink; kit.c.icon_muted = color(theme.muted);
            kit.c.border_base = color(theme.border); kit.c.border_muted = color(theme.border);
            kit.c.layer_01 = color(theme.surface); kit.c.surface_base = color(theme.surface);
            let hints: Vec<_> = n.hint.split('\n').collect();
            let trigger = Combobox::new(format!("choice-{id}"), &s.combos[&id], &options,
                move |s: &mut State| s.combos.get_mut(&id).unwrap())
                .current(Some(&n.choice.to_string()))
                .label(hints.first().copied().unwrap_or(""))
                .search_placeholder(hints.get(1).copied().unwrap_or("Search"))
                .empty_message(hints.get(2).copied().unwrap_or("No results"))
                .width(if st.width >= 0.0 { st.width } else { 280.0 })
                .on_choose(move |s: &mut State, value, _| {
                    if let Ok(index) = value.parse() { s.emit(18, id, index, String::new()); }
                }).build(&kit, layers);
            // Keep the kit trigger key: the popup anchors to it by name.
            // The public node key and layout belong on a separate wrapper.
            rune::el().child(trigger)
        },
        7 => n.document.as_ref().map_or_else(||col(),|d|d.view(id,st.font_size,ink,&theme)),
        _ => unreachable!(),
    }.key(format!("node-{id}"));
    // Composite checkboxes need a default internal label gap even when the
    // caller leaves the generic container gap at its default zero.
    let gap = if n.kind == 6 && st.gap == 0.0 { 12.0 } else { st.gap };
    e = e.padding(st.padding).gap(gap).radius(st.radius).fill(color(st.background_rgba));
    if st.flags & 256 != 0 {e=e.border(theme.border_width,color(theme.border)).border_layout();}
    if n.kind != 9 && !n.hint.is_empty(){e=e.help(n.hint.clone()).label(n.hint.clone());}
    if n.kind == 4 && s.tree.theme.is_some() {
        e=e.hover(ink.alpha(0.065)).press(ink.alpha(0.10)).press_scale(0.97).cursor(rune::Cursor::Pointer);
    } else if n.kind == 4 {
        e = e.border(1.0, Color::WHITE.alpha(0.12))
            .hover(Color::WHITE.alpha(0.07)).press(Color::BLACK.alpha(0.18))
            .cursor(rune::Cursor::Pointer);
        if st.background_rgba & 255 > 32 {
            let fill = color(st.background_rgba);
            e = e.gradient(fill.mix(Color::WHITE.alpha(fill.a), 0.035), fill.mix(Color::BLACK.alpha(fill.a), 0.06))
                .shadow(1.0, 3.0, Color::BLACK.alpha(0.16));
        }
    }
    if st.radius>0.0 && n.kind<=2 {e=e.clip_rounded(st.radius);}
    if n.kind == 6 { e = e.cursor(rune::Cursor::Pointer); }
    if st.width >= 0.0 { e = if st.flags & 128 != 0 {e.max_w(st.width)}else{e.w(st.width)}; }
    if st.flags & 64 != 0 {e=e.items_center();}
    if st.height >= 0.0 { e = e.h(st.height); }
    if st.flags & 1 != 0 { e = e.grow(1.0).min_w(0.0).min_h(0.0); }
    if st.flags & 2 != 0 { e = e.w_full(); }
    if st.flags & 4 != 0 { e = e.h_full(); }
    if st.flags & 8 != 0 { e = e.scroll(ScrollSpec { stick_bottom: false }).min_h(0.0); }
    if st.flags & 16 != 0 { e = e.wrap(); }
    if st.flags & 32 != 0 { e = e.motion(Spring::new(0.25, 0.0)).transition(Transition::opacity()); }
    if n.kind == 4 {
        if let Some(fx) = n.effect {
            e = e.child(rune::effect(id).key("shader").absolute(0.0, 0.0)
                .w_full().h_full().radius(st.radius).inert()
                .effect_bleed((fx.size * 4.0).max(64.0)).effect_backdrop(fx.kind == 8));
        }
        let label = text(n.text.clone()).font(font).color(ink);
        // A moving reflective surface can pass through the label's luminance.
        // Keep a stable dark backing for the default light ink on these materials.
        if n.effect.is_some_and(|fx| matches!(fx.kind, 4 | 5)) && ink.luminance() > 0.4 {
            e = e.child(row().px(8.0).py(3.0).radius(5.0).fill(Color::BLACK.alpha(0.48)).child(label));
        } else { e = e.child(label); }
    }
    if n.kind==4 && st.flags&1024!=0 {e=e.default_button();}
    if st.flags&512!=0 {
        e=e.role("dialog").label(n.text.clone()).modal(true,move|s:&mut State|s.emit(16,id,0,String::new()))
            .shadow(8.0,28.0,Color::BLACK.alpha(0.24));
    }
    for child in &n.children {
        if s.tree.nodes[child].style.flags&512!=0 {
            // Out of normal flow: Rune traps focus, blocks background input and
            // restores focus to the opener when the host removes this panel.
            e=e.child(rune::el().key("modal-scrim").absolute(0.0,0.0).w_full().h_full().fill(Color::BLACK.alpha(0.40)).a11y_hidden())
                .child(rune::el().key("modal-layer").absolute(0.0,0.0).w_full().h_full().padding(20.0).center().child(node(s,*child,layers)));
        } else { e = e.child(node(s, *child,layers)); }
    }
    if st.flags & 8 != 0 {e=e.scroll_bar_styled(ScrollBarStyle::neutral(ink.luminance()>0.4).width(4.0).hover_width(6.0));}
    e
}

pub fn view(s: &State, _: &mut Cx<State>) -> El {
    let mut layers = Vec::new();
    let base = node(s, s.tree.root, &mut layers);
    rune_kit::root(base, layers)
}
