// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
use super::*;
#[test]fn formatted_extension_fragment_is_native(){
 let d=Document::new("<h1>Été</h1><p><strong>bold</strong> <del>old</del> <a href=\"next.md\">link</a></p><table><tr><th>A</th><th>B</th></tr><tr><td>x</td><td>y</td></tr></table><ul><li><input checked disabled type=\"checkbox\"> done</li></ul>","","").unwrap();
 assert_eq!(d.root.children[0].tag,"h1");assert_eq!(d.root.children[2].tag,"table");
 assert_eq!(decode("&eacute; &#x1F680; &#233; &unknown;"),"é 🚀 é &unknown;");
 let _=d.view(1,16.0,rune::Color::WHITE,&crate::ui_bridge::RuneUiTheme::preset(true));
}
#[test]fn bounded_and_quoted_markup(){
 let e=parse("<p title=\"a > b\">ok<br/>next</p>").unwrap();assert_eq!(e.children[0].attr("title"),"a > b");
 assert!(parse(&"<div>".repeat(70)).is_err());assert!(parse(&"x".repeat(2*1024*1024+1)).is_err());
}

#[cfg(target_os="windows")]
#[test]
fn installed_monospace_has_equal_advances(){
    let mut ts=rune::text::TextSystem::new(&[],("Segoe UI Variable Text, Segoe UI, Inter","Cascadia Mono, Consolas, Courier New"));
    let font=rune::TextStyle{family:rune::Family::Mono,..rune::TextStyle::new(16.0,400.0)};
    let narrow=ts.measure("iiiiiiii",font,1.0).0;
    let wide=ts.measure("WWWWWWWW",font,1.0).0;
    assert!(narrow>0.0);
    assert!((narrow-wide).abs()<0.1,"Monospace glyph advances differ: {narrow} vs {wide}");
    let body=rune::TextStyle::new(17.0,400.0);
    assert!(ts.measure("WWWWWWWW",body,1.0).0-ts.measure("iiiiiiii",body,1.0).0>10.0);
}
