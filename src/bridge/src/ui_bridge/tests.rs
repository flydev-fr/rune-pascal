// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
use super::model::*;
use super::view;
use std::sync::Arc;

#[test]
fn invalid_tree_operations_are_atomic_and_depth_is_bounded() {
    let mut t = Tree::default();
    assert!(t.add(0, 1, 1, String::new()).is_ok());
    assert!(t.add(1, 2, 3, "label".into()).is_ok());
    assert!(t.add(2, 3, 3, String::new()).is_err());
    assert!(t.add(999, 3, 1, String::new()).is_err());
    assert!(t.add(1, 2, 1, String::new()).is_err());
    assert!(t.add(0, 4, 1, String::new()).is_err());
    assert_eq!(t.nodes.len(), 2);
    assert_eq!(t.nodes[&1].children, vec![2]);
    let mut parent = 1;
    for id in 10..74 { t.add(parent, id, 1, String::new()).unwrap(); parent = id; }
    assert!(t.add(parent, 99, 1, String::new()).is_err());
}

#[test]
fn style_layout_and_validation() {
    assert_eq!(size_of::<RuneUiStyle>(), 44);
    assert_eq!(std::mem::offset_of!(RuneUiStyle, background_rgba), 36);
    let mut style = RuneUiStyle::default();
    assert!(style.valid());
    style.padding = f32::NAN;
    assert!(!style.valid());
    style = RuneUiStyle::default(); style.flags = 2048;
    assert!(!style.valid());
}

#[test]
fn native_controls_emit_utf8_click_and_controlled_checkbox_events() {
    let mut tree = Tree::default();
    tree.add(0, 1, 1, String::new()).unwrap();
    tree.nodes.get_mut(&1).unwrap().style = RuneUiStyle {
        flags: 6, padding: 12.0, gap: 8.0, ..RuneUiStyle::default()
    };
    for (id, kind, label) in [(2, 5, ""), (3, 4, "Add"), (4, 6, "Task")] {
        tree.add(1, id, kind, label.into()).unwrap();
        let n = tree.nodes.get_mut(&id).unwrap(); n.style.width = 250.0; n.style.height = 40.0;
    }
    tree.nodes.get_mut(&3).unwrap().effect = Some(super::effects::RuneUiEffect::preset(1));
    let mut rt = rune::Runtime::new(State::new(Arc::new(tree)), view::view, (400.0, 300.0), rune::Env::new(true, 1.0));
    rt.frame(0.0);
    assert!(rt.instances.iter().any(|instance| instance.params[1] == rune::render::KIND_EFFECT));
    let (x, y) = rt.find("node-2").unwrap();
    rt.pointer_down(x, y); rt.pointer_up(x, y); rt.frame(0.0);
    rt.paste("café ✓"); rt.frame(0.0);
    assert_eq!(rt.state.edits[&2].text, "café ✓");
    assert!(rt.state.events.borrow().iter().any(|e| e.kind == 11 && e.id == 2 && e.text == "café ✓"));
    rt.key(rune::KeyEvent { key: rune::Key::Enter, mods: rune::Mods::default() });
    rt.frame(0.0);
    assert!(rt.state.events.borrow().iter().any(|e| e.kind == 12 && e.id == 2 && e.text == "café ✓"));
    // Host commits the same text: preserve the focused editor and its selection.
    let mut updated = (*rt.state.tree).clone();
    updated.nodes.get_mut(&2).unwrap().text = "café ✓".into();
    let before = rt.state.edits[&2].clone();
    rt.state.install(Arc::new(updated));
    assert_eq!(rt.state.edits[&2], before);
    rt.frame(0.0);
    for (id, kind) in [(3, 10), (4, 13)] {
        let (x, y) = rt.find(&format!("node-{id}")).unwrap();
        rt.pointer_down(x, y); rt.pointer_up(x, y); rt.frame(0.0);
        assert!(rt.state.events.borrow().iter().any(|e| e.id == id && e.kind == kind));
    }
    assert!(rt.state.events.borrow().iter().any(|e| e.kind == 13 && e.value == 1));
    assert!(!rt.state.tree.nodes[&4].checked); // host must explicitly accept CHECK
}

#[test]
fn effect_abi_bounds_and_idle_layers() {
    use super::effects::RuneUiEffect;
    assert_eq!(size_of::<RuneUiEffect>(), 32);
    assert_eq!(std::mem::offset_of!(RuneUiEffect, intensity), 16);
    let shape = rune_fx::Shape::rounded(0.0, 0.0, 160.0, 48.0, 10.0);
    for kind in 0..=8 {
        let mut effect = RuneUiEffect::preset(kind);
        assert!(effect.valid());
        effect.flags = 2;
        assert!(!effect.layer(shape, 2.0).is_animating());
        effect.flags = 0;
        effect.intensity = 0.0;
        assert!(!effect.layer(shape, 2.0).is_animating());
        effect.speed = f32::NAN;
        assert!(!effect.valid());
    }
    let mut bad = RuneUiEffect::preset(9);
    assert!(!bad.valid());
    bad = RuneUiEffect::preset(1); bad.flags = 4;
    assert!(!bad.valid());
    bad.flags = 0; bad.size = 129.0;
    assert!(!bad.valid());
}

#[test]
fn compact_recent_close_is_a_separate_click_target() {
 let mut t=Tree::default();t.theme=Some(super::RuneUiTheme::preset(true));
 t.add(0,1,1,String::new()).unwrap();t.nodes.get_mut(&1).unwrap().style.flags=6;
 t.add(1,2,2,String::new()).unwrap();t.nodes.get_mut(&2).unwrap().style=RuneUiStyle{width:234.0,height:36.0,gap:3.0,..RuneUiStyle::default()};
 t.add(2,1000,8,"A long document title which must leave the close button visible".into()).unwrap();
 t.nodes.get_mut(&1000).unwrap().style=RuneUiStyle{flags:1,height:34.0,padding:8.0,font_size:12.0,..RuneUiStyle::default()};
 t.add(2,10000,4,"×".into()).unwrap();t.nodes.get_mut(&10000).unwrap().style=RuneUiStyle{width:24.0,height:28.0,..RuneUiStyle::default()};
 let mut rt=rune::Runtime::new(State::new(Arc::new(t)),view::view,(280.0,180.0),rune::Env::new(true,1.0));rt.frame(0.0);
 let (x,y)=rt.find("node-10000").unwrap();assert!(x<234.0 && x>200.0);
 rt.pointer_down(x,y);rt.pointer_up(x,y);rt.frame(0.0);
 let events=rt.state.events.borrow();assert_eq!(events.len(),1);assert_eq!(events[0].id,10000);
}

#[test]
fn bordered_reading_panel_reserves_its_full_perimeter() {
 for scale in [1.0,1.25,1.5,2.0] {
  let mut t=Tree::default();t.theme=Some(super::RuneUiTheme::preset(true));
  t.add(0,1,1,String::new()).unwrap();t.nodes.get_mut(&1).unwrap().style=RuneUiStyle{flags:6|256,radius:14.0,..RuneUiStyle::default()};
  t.add(1,2,1,String::new()).unwrap();t.nodes.get_mut(&2).unwrap().style=RuneUiStyle{flags:1|2,background_rgba:0x191919ff,..RuneUiStyle::default()};
  let mut rt=rune::Runtime::new(State::new(Arc::new(t)),view::view,(400.0,300.0),rune::Env::new(true,scale));rt.frame(0.0);
  let (outer,_)=rt.presented("node-1").unwrap();let (inner,_)=rt.presented("node-2").unwrap();
  assert!(inner[0]>outer[0] && inner[1]>outer[1],"child covers top/left border at scale {scale}");
  assert!(inner[0]+inner[2]<outer[0]+outer[2] && inner[1]+inner[3]<outer[1]+outer[3],"child covers bottom/right border at scale {scale}");
 }
}

#[test]
fn themed_modal_traps_input_and_defaults_to_cancel() {
 let mut t=Tree::default();t.theme=Some(super::RuneUiTheme::preset(true));
 t.add(0,1,1,String::new()).unwrap();t.nodes.get_mut(&1).unwrap().style.flags=6;
 t.add(1,99,4,"Background".into()).unwrap();t.nodes.get_mut(&99).unwrap().style=RuneUiStyle{width:140.0,height:32.0,..RuneUiStyle::default()};
 t.add(1,700,1,"Remove folder?".into()).unwrap();t.nodes.get_mut(&700).unwrap().style=RuneUiStyle{width:460.0,padding:22.0,gap:12.0,radius:14.0,flags:512|256,..RuneUiStyle::default()};
 for (id,label) in [(27,"Cancel"),(28,"Remove")] {t.add(700,id,4,label.into()).unwrap();t.nodes.get_mut(&id).unwrap().style=RuneUiStyle{width:100.0,height:30.0,..RuneUiStyle::default()};}
 t.nodes.get_mut(&27).unwrap().style.flags=1024;
 let mut rt=rune::Runtime::new(State::new(Arc::new(t)),view::view,(800.0,600.0),rune::Env::new(true,1.0));rt.frame(0.0);
 let (panel,_)=rt.presented("node-700").unwrap();assert!((panel[0]+panel[2]/2.0-400.0).abs()<0.1);assert!((panel[1]+panel[3]/2.0-300.0).abs()<0.1);
 assert!(rt.elements().iter().any(|e|e.focused && e.key.as_deref()==Some("node-27")));
 rt.key(rune::KeyEvent{key:rune::Key::Enter,mods:Default::default()});rt.frame(0.0);
 assert!(rt.state.events.borrow().iter().any(|e|e.id==27 && e.kind==10));
 rt.state.events.borrow_mut().clear();
 let (x,y)=rt.find("node-99").unwrap();rt.pointer_down(x,y);rt.pointer_up(x,y);rt.frame(0.0);
 assert!(!rt.state.events.borrow().iter().any(|e|e.id==99));
 assert!(rt.state.events.borrow().iter().any(|e|e.id==700 && e.kind==16));
 rt.state.events.borrow_mut().clear();
 rt.key(rune::KeyEvent{key:rune::Key::Escape,mods:Default::default()});rt.frame(0.0);
 assert!(rt.state.events.borrow().iter().any(|e|e.id==700 && e.kind==16));
 rt.state.events.borrow_mut().clear();let (x,y)=rt.find("node-28").unwrap();rt.pointer_down(x,y);rt.pointer_up(x,y);rt.frame(0.0);
 assert!(rt.state.events.borrow().iter().any(|e|e.id==28 && e.kind==10));
}

#[test]
fn combobox_unicode_choice_keyboard_and_escape() {
    let mut t=Tree::default();
    t.theme=Some(super::RuneUiTheme::preset(true));
    t.add(0,1,1,String::new()).unwrap(); t.nodes.get_mut(&1).unwrap().style.flags=6;
    t.add(1,2,1,String::new()).unwrap();
    t.nodes.get_mut(&1).unwrap().style.padding=80.0;
    t.nodes.get_mut(&2).unwrap().style=RuneUiStyle{height:44.0,radius:12.0,..RuneUiStyle::default()};
    t.add(2,805,9,"English\nFrançais\nDeutsch\n简体中文\nРусский".into()).unwrap();
    t.nodes.get_mut(&805).unwrap().style.width=300.0;
    let mut rt=rune::Runtime::new(State::new(Arc::new(t)),view::view,(600.0,420.0),rune::Env::new(true,1.0));
    rt.frame(0.0);
    let (x,y)=rt.find("node-805").unwrap(); rt.pointer_down(x,y); rt.pointer_up(x,y); rt.frame(0.0);
    assert!(rt.state.combos[&805].open);
    let (trigger,_) = rt.presented("node-805").unwrap();
    let (popup,_) = rt.presented("choice-805-content").unwrap();
    assert!((popup[0]-trigger[0]).abs()<2.0,"popup {popup:?} must align with {trigger:?}");
    assert!(popup[1]>=trigger[1]+trigger[3],"popup {popup:?} must appear below {trigger:?}");
    rt.paste("Рус"); rt.frame(0.0);
    rt.key(rune::KeyEvent{key:rune::Key::Enter,mods:Default::default()}); rt.frame(0.0);
    assert!(!rt.state.combos[&805].open);
    assert!(rt.state.events.borrow().iter().any(|e|e.kind==18 && e.id==805 && e.value==4));
    assert_eq!(rt.state.tree.nodes[&805].choice,0);
    let mut updated=(*rt.state.tree).clone();updated.nodes.get_mut(&805).unwrap().choice=4;
    rt.state.install(Arc::new(updated));rt.frame(0.0);
    rt.state.events.borrow_mut().clear();
    let (x,y)=rt.find("node-805").unwrap();rt.pointer_down(x,y);rt.pointer_up(x,y);rt.frame(0.0);
    assert!(rt.state.combos[&805].open);
    rt.key(rune::KeyEvent{key:rune::Key::Escape,mods:Default::default()});rt.frame(0.0);
    assert!(!rt.state.combos[&805].open);assert!(rt.state.events.borrow().is_empty());
    let (x,y)=rt.find("node-805").unwrap();rt.pointer_down(x,y);rt.pointer_up(x,y);rt.frame(0.0);
    // Let the popup entry animation finish before a pointer hit test.
    for _ in 0..30 { rt.frame(1.0); }
    let (x,y)=rt.find("choice-805-option-2").unwrap();
    rt.pointer_down(x,y);rt.pointer_up(x,y);rt.frame(0.0);
    assert!(!rt.state.combos[&805].open);
    let events=rt.state.events.borrow();assert_eq!(events.len(),1);assert_eq!(events[0].kind,18);assert_eq!(events[0].value,2);
}

#[test]
fn combobox_rejects_empty_and_excess_options_atomically() {
    let mut t=Tree::default();t.add(0,1,1,String::new()).unwrap();
    for bad in ["".to_owned(),"English\n".to_owned(),"English\n  ".to_owned(),vec!["option";65].join("\n")] {
        assert!(t.add(1,805,9,bad).is_err());
        assert_eq!(t.nodes.len(),1);assert!(t.nodes[&1].children.is_empty());
    }
    t.add(1,805,9,"English\nFrançais".into()).unwrap();
}


#[test]
fn combobox_anchor_survives_display_scaling() {
    for scale in [1.0,1.25,1.5,2.0] {
        let mut t=Tree::default();t.add(0,1,1,String::new()).unwrap();
        t.nodes.get_mut(&1).unwrap().style=RuneUiStyle{flags:6,padding:48.0,..RuneUiStyle::default()};
        t.add(1,805,9,"English\nFrançais\nDeutsch\n简体中文\nРусский".into()).unwrap();
        t.nodes.get_mut(&805).unwrap().style.width=280.0;
        let mut rt=rune::Runtime::new(State::new(Arc::new(t)),view::view,(800.0,500.0),rune::Env::new(true,scale));
        rt.state.combos.get_mut(&805).unwrap().open=true;rt.frame(0.0);
        let (trigger,_)=rt.presented("node-805").unwrap();let (popup,_)=rt.presented("choice-805-content").unwrap();
        assert!((trigger[0]-popup[0]).abs()<2.0,"scale {scale}: {trigger:?} {popup:?}");
        assert!(popup[1]>=trigger[1]+trigger[3]);
    }
}
