// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
use super::*;
#[test]
fn abi_and_rejected_arguments() {
    assert_eq!(size_of::<RuneHostFrame>(),40);
    assert_eq!(size_of::<RuneBenchResult>(),48);
    assert!(!valid_size(0,260,1.0));
    assert!(!valid_size(700,16385,1.0));
    assert!(!valid_size(700,260,f32::NAN));
    assert!(!valid_size(700,260,0.0));
    assert!(valid_size(875,325,1.25));
    assert!(with_host(0,|_|Ok(())).is_err());
    assert_eq!(rune_host_destroy(0),-1);
    assert_eq!(unsafe{rune_host_frame(0,std::ptr::null_mut())},-1);
    let mut id=99;
    assert_eq!(unsafe{rune_host_create(0,700,260,1.0,None,std::ptr::null_mut(),&mut id)},-1);
}
#[test]
fn ids_are_unique_across_ui_threads() {
    let a=std::thread::spawn(next_id).join().unwrap().unwrap();
    let b=std::thread::spawn(next_id).join().unwrap().unwrap();
    assert_ne!(a,b);
    assert_ne!(a,0);
}

#[test]
fn document_copy_exports_unicode_and_indented_code_once(){
    let mut tree=super::super::model::Tree::default();
    tree.add(0,1,1,String::new()).unwrap();tree.nodes.get_mut(&1).unwrap().style.flags=6;
    tree.add(1,2,7,String::new()).unwrap();
    tree.nodes.get_mut(&2).unwrap().document=Some(std::sync::Arc::new(super::super::document::Document::new(
        "<p>été <strong>café</strong> 🦀</p><pre><code>  x := 1;\n  y := 2;</code></pre>","","").unwrap()));
    let mut rt=rune::Runtime::new(State::new(std::sync::Arc::new(tree)),view::view,(800.0,500.0),rune::Env::new(true,1.0));rt.frame(0.0);
    key_input(&mut rt,67,2).unwrap();rt.frame(0.0);assert!(take_copy_event(&mut rt).is_none());
    key_input(&mut rt,65,2).unwrap();rt.frame(0.0);
    assert!(rt.selected_text().is_some());
    key_input(&mut rt,67,2).unwrap();rt.frame(0.0);
    let event=take_copy_event(&mut rt).expect("copy event");assert_eq!(event.kind,17);
    assert!(event.text.contains("été café 🦀"));assert!(event.text.contains("  x := 1;\n  y := 2;"));assert!(!event.text.contains("<strong>"));
    rt.frame(0.0);assert!(take_copy_event(&mut rt).is_none());
}
#[test]
fn focused_input_copy_uses_the_same_embedded_key_path(){
    let mut tree=super::super::model::Tree::default();tree.add(0,1,1,String::new()).unwrap();
    tree.add(1,2,5,"Crème 👋".into()).unwrap();tree.nodes.get_mut(&2).unwrap().style.width=240.0;tree.nodes.get_mut(&2).unwrap().style.height=32.0;
    let mut rt=rune::Runtime::new(State::new(std::sync::Arc::new(tree)),view::view,(400.0,200.0),rune::Env::new(true,1.0));rt.frame(0.0);
    let(x,y)=rt.find("node-2").unwrap();rt.pointer_down(x,y);rt.pointer_up(x,y);rt.frame(0.0);
    key_input(&mut rt,65,2).unwrap();rt.frame(0.0);key_input(&mut rt,67,2).unwrap();rt.frame(0.0);
    assert_eq!(take_copy_event(&mut rt).unwrap().text,"Crème 👋");
    assert!(take_copy_event(&mut rt).is_none());
}
