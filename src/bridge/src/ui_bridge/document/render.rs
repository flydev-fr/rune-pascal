// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
use super::*;
use crate::ui_bridge::model::State;
use rune::{El,Color,Span,Link,TextStyle,Family,col,row,text,rich,el};
fn rgba(n:u32)->Color{crate::ui_bridge::view::color(n)}
use crate::ui_bridge::RuneUiTheme;
#[derive(Clone,Default)]struct Format{bold:bool,italic:bool,mono:bool,strike:bool,under:bool,mark:bool,small:bool,link:String}
struct Run{value:String,f:Format}
fn inline(e:&Element,f:&Format,runs:&mut Vec<Run>){
    let mut f=f.clone();match e.tag.as_str(){
        "strong"|"b"|"th"=>f.bold=true,"em"|"i"=>f.italic=true,"code"|"kbd"|"samp"=>f.mono=true,
        "del"|"s"|"strike"=>f.strike=true,"ins"|"u"=>f.under=true,"mark"=>f.mark=true,
        "sup"|"sub"=>f.small=true,"a"=>f.link=e.attr("href").into(),
        "br"=>{runs.push(Run{value:"\n".into(),f});return},
        "input"=>{runs.push(Run{value:if e.has("checked"){"☑  ".into()}else{"☐  ".into()},f});return},
        "script"|"style"=>return,_=>{}
    }
    if !e.text.is_empty(){runs.push(Run{value:e.text.clone(),f:f.clone()})}
    for c in &e.children{inline(c,&f,runs)}
}
fn para(e:&Element,id:u32,font:f32,ink:Color,theme:&RuneUiTheme,query:&str)->El{
    let mut runs=Vec::new();inline(e,&Format{bold:matches!(e.tag.as_str(),"h1"|"h2"|"h3"|"h4"|"h5"|"h6"),..Format::default()},&mut runs);
    let mut value=String::new();let mut spans=Vec::new();let mut links=Vec::new();let mut strikes=Vec::new();
    for r in runs{
        let a=value.len();value.push_str(&r.value);let b=value.len();if a==b{continue}
        let mut sp=Span::new(a,b);if r.f.bold{sp.weight=Some(650.0)}if r.f.italic{sp.italic=true}
        if r.f.mono{sp.mono=true;sp.size=Some(font*0.94);sp.bg=Some(rgba(theme.inline_code_background));sp.bg_pad=theme.inline_code_padding}
        if r.f.small{sp.size=Some(font*0.76)}
        if r.f.under{sp.underline=Some(ink)}if r.f.mark{sp.bg=Some(rgba(theme.highlight))}
        if r.f.strike{strikes.push((a,b,None))}
        if !r.f.link.is_empty(){sp.color=Some(rgba(theme.link));sp.underline=sp.color;links.push(Link::new(a..b,r.f.link))}
        spans.push(sp);
    }
    let mut p=rich(value.clone(),spans).strikes(strikes).font(TextStyle{line_height:font*1.62,..TextStyle::new(font,400.0)}).color(ink).w_full().wrap().selectable(true);
    for l in links{p=p.link(l)}
    p=p.on_link(move|s:&mut State,url,_|s.emit(15,id,0,url.to_owned()));
    if !query.is_empty(){
        let needle=query.to_lowercase();let mut highlights=Vec::new();
        // Character-aware matching keeps UTF-8 byte ranges valid for accented searches.
        let count=needle.chars().count();
        for (a,_) in value.char_indices(){
            let end=value[a..].char_indices().nth(count).map_or(value.len(),|(b,_)|a+b);
            if end>a && value[a..end].to_lowercase()==needle {
                highlights.push(rune::TextHighlight::new(a..end,rgba(theme.highlight)));
            }
        }
        p=p.text_highlights(highlights);
    }p
}
fn plain(e:&Element)->String{let mut out=e.text.clone();for c in &e.children{out.push_str(&plain(c))}out}
fn block(e:&Element,d:&Document,id:u32,font:f32,ink:Color,theme:&RuneUiTheme,depth:usize)->El{
    let bg=rgba(theme.code_background);let line=rgba(theme.border);
    match e.tag.as_str(){
        "script"|"style"|"head"=>el().h(0.0),
        "h1"|"h2"|"h3"|"h4"|"h5"|"h6"=>{
            let level=e.tag[1..].parse::<u32>().unwrap_or(2);let size=font*match level{1=>2.0,2=>1.5,3=>1.22,_=>1.05};
            para(e,id,size,ink,theme,&d.query).role("heading").level(level).mt(if level==1{0.0}else{14.0}).mb(8.0)
        },
        "p"=>{
            let mut p=col().w_full().gap(8.0).child(para(e,id,font,ink,theme,&d.query));
            for image in e.children.iter().filter(|c|c.tag=="img"){p=p.child(block(image,d,id,font,ink,theme,depth+1))}p.mb(12.0)
        },
        "pre"=>text(plain(e).trim_end_matches('\n').to_owned()).font(TextStyle{family:Family::Mono,line_height:font*1.5,..TextStyle::new(font*0.88,400.0)}).color(ink).selectable(true).wrap().w_full().padding(theme.code_padding).fill(bg).border(theme.border_width,rgba(theme.code_border)).radius(theme.code_radius).mb(14.0),
        "hr"=>el().h(1.0).w_full().fill(line).my(14.0),
        "blockquote"=>row().w_full().gap(16.0).child(el().w(3.0).h_full().fill(rgba(theme.quote_border))).child(children(e,d,id,font,ink,theme,depth+1).grow(1.0).min_w(0.0)).padding(12.0).fill(rgba(theme.quote_background)).radius(theme.code_radius).mb(14.0),
        "ul"|"ol"=>{
            let mut list=col().w_full().gap(5.0).mb(12.0);let ordered=e.tag=="ol";
            for (i,child) in e.children.iter().filter(|c|c.tag=="li").enumerate(){
                let mut runs=Vec::new();inline(child,&Format::default(),&mut runs);let task=runs.first().is_some_and(|r|r.value.starts_with('☑')||r.value.starts_with('☐'));
                let marker=if task{"".into()}else if ordered{format!("{}.",i+1)}else{"•".into()};
                let content=if child.children.iter().any(|c|matches!(c.tag.as_str(),"p"|"ul"|"ol")){children(child,d,id,font,ink,theme,depth+1)}else{para(child,id,font,ink,theme,&d.query)};
                list=list.child(row().w_full().gap(8.0).child(text(marker).font(TextStyle::new(font,400.0)).color(ink).w(if task{0.0}else{20.0})).child(content.grow(1.0).min_w(0.0)))
            }list
        },
        "table"=>{
            fn rows<'a>(e:&'a Element,out:&mut Vec<&'a Element>){if e.tag=="tr"{out.push(e)}else{for c in &e.children{rows(c,out)}}}
            let mut rs=Vec::new();rows(e,&mut rs);let mut table=col().w_full().border(theme.border_width,line).radius(theme.code_radius).mb(16.0);
            for (n,r) in rs.iter().enumerate(){let mut cells=row().w_full().gap(0.0);for c in r.children.iter().filter(|c|c.tag=="td"||c.tag=="th"){
                cells=cells.child(para(c,id,font*0.9,ink,theme,&d.query).padding(11.0).grow(1.0).min_w(0.0));
            }if n==0||n%2==0{cells=cells.fill(rgba(theme.table_header))}table=table.child(cells)}table
        },
        "img"=>{
            let alt=e.attr("alt");if let Some(i)=d.images.get(e.attr("src")){let w=i.size().map_or(600,|s|s[0]);el().w_full().max_w((w as f32).min(900.0)).pixels(i.pixels()).radius(6.0).role("image").label(alt).mb(12.0)}
            else{text(format!("▧  {}",if alt.is_empty(){e.attr("src")}else{alt})).font(TextStyle::new(font*0.88,400.0)).color(ink).padding(12.0).fill(bg).radius(6.0)}
        },
        ""=>if e.text.trim().is_empty(){el().h(0.0)}else{para(e,id,font,ink,theme,&d.query)},
        "div"|"root"|"section"|"article"|"body"|"li"|"details"|"summary"=>children(e,d,id,font,ink,theme,depth+1),
        _=>para(e,id,font,ink,theme,&d.query)
    }
}
fn children(e:&Element,d:&Document,id:u32,font:f32,ink:Color,theme:&RuneUiTheme,depth:usize)->El{
    let mut flow=col().w_full().min_w(0.0);
    for (n,c) in e.children.iter().enumerate(){if c.tag.is_empty()&&c.text.trim().is_empty(){continue}flow=flow.child(block(c,d,id,font,ink,theme,depth+1).key(format!("b{n}")))}flow
}
pub fn view(d:&Document,id:u32,font:f32,ink:Color,theme:&RuneUiTheme)->El{children(&d.root,d,id,font,ink,theme,0).selection_group(true)}
