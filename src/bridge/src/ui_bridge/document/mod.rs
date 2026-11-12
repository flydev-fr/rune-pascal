// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
//! Native Rune document flow fed by mormot.ext.markdown's formatted fragment.
//! This is a bounded document adapter, not a browser, CSS engine or Markdown parser.
mod parse;
mod render;
use std::collections::HashMap;
pub use parse::{Element,parse,decode};
pub struct Document { pub root:Element,pub images:HashMap<String,rune::Image>,pub query:String }
impl Document {
    pub fn new(markup:&str,base:&str,query:&str)->Result<Self,String>{
        let root=parse(markup)?;
        let mut sources=Vec::new();
        fn collect(e:&Element,out:&mut Vec<String>){if e.tag=="img"{out.push(e.attr("src").to_owned())}for c in &e.children{collect(c,out)}}
        collect(&root,&mut sources);
        thread_local! {static CACHE:rune::ImageCache=rune::ImageCache::new(std::sync::Arc::new(||{}));}
        let mut images=HashMap::new();
        for src in sources.into_iter().take(24){
            // File previews never trigger a network fetch.
            if src.contains("://")||src.starts_with("data:"){continue}
            let path=std::path::Path::new(base).join(&src);
            if path.metadata().is_ok_and(|m|m.len()<=8*1024*1024){
                let image=CACHE.with(|cache|cache.file(path,900.0,1.0));
                if image.wait().is_ok(){images.insert(src,image);}
            }
        }
        Ok(Self{root,images,query:query.to_owned()})
    }
    pub fn view(&self,id:u32,font:f32,ink:rune::Color,theme:&super::RuneUiTheme)->rune::El {render::view(self,id,font,ink,theme)}
}
#[cfg(test)]mod tests;
