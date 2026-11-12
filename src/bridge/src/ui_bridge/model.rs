// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
use std::collections::{BTreeMap, HashMap};
use std::cell::RefCell;
use std::sync::Arc;

pub const MAX_NODES: usize = 2048;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RuneUiStyle {
    pub struct_size: u32,
    pub flags: u32,
    pub width: f32,
    pub height: f32,
    pub gap: f32,
    pub padding: f32,
    pub radius: f32,
    pub font_size: f32,
    pub font_weight: u32,
    pub background_rgba: u32,
    pub foreground_rgba: u32,
}
impl Default for RuneUiStyle {
    fn default() -> Self {
        Self { struct_size: size_of::<Self>() as u32, flags: 0, width: -1.0, height: -1.0,
            gap: 0.0, padding: 0.0, radius: 0.0, font_size: 14.0, font_weight: 400,
            background_rgba: 0, foreground_rgba: 0xe8eaf0ff }
    }
}
impl RuneUiStyle {
    pub fn valid(&self) -> bool {
        self.struct_size == size_of::<Self>() as u32 && self.flags & !2047 == 0
        && [self.width, self.height].iter().all(|v| v.is_finite() && *v >= -1.0 && *v <= 8192.0)
        && [self.gap, self.padding, self.radius].iter().all(|v| v.is_finite() && *v >= 0.0 && *v <= 1024.0)
        && self.font_size.is_finite() && (6.0..=256.0).contains(&self.font_size)
        && (100..=900).contains(&self.font_weight)
    }
}

#[derive(Clone)]
pub struct Node {
    pub id: u32,
    pub kind: u32,
    pub text: String,
    pub hint: String,
    pub checked: bool,
    pub choice: u32,
    pub style: RuneUiStyle,
    pub children: Vec<u32>,
    pub effect: Option<super::effects::RuneUiEffect>,
    pub document: Option<Arc<super::document::Document>>,
    depth: usize,
}
#[derive(Clone, Default)]
pub struct Tree {
    pub theme: Option<super::theme::RuneUiTheme>,
    pub root: u32,
    pub nodes: BTreeMap<u32, Node>,
    pub revision: u32,
    bytes: usize,
}
impl Tree {
    pub fn add(&mut self, parent: u32, id: u32, kind: u32, text: String) -> Result<(), String> {
        if id == 0 || self.nodes.contains_key(&id) || !(1..=9).contains(&kind) {
            return Err("Invalid/duplicate node ID or kind".into());
        }
        if self.nodes.len() >= MAX_NODES || self.bytes + text.len() > 2 * 1024 * 1024 {
            return Err("UI tree limit exceeded (2048 nodes / 2 MiB text)".into());
        }
        if kind == 9 && (text.split('\n').count() > 64 || text.split('\n').any(|s| s.trim().is_empty())) {
            return Err("Combobox needs 1..64 nonempty options".into());
        }
        let depth = if parent == 0 {
            if self.root != 0 || kind > 2 { return Err("One row/column root is required".into()); }
            0
        } else {
            let p = self.nodes.get(&parent).ok_or("Parent must exist before its child")?;
            if p.kind > 2 { return Err("Only rows/columns may have children".into()); }
            p.depth + 1
        };
        if depth > 64 { return Err("UI depth limit exceeded (64)".into()); }
        self.bytes += text.len();
        self.nodes.insert(id, Node { id, kind, text, hint: String::new(), checked: false, choice: 0, style: RuneUiStyle::default(), children: vec![], effect: None, document: None, depth });
        if parent == 0 { self.root = id; } else { self.nodes.get_mut(&parent).unwrap().children.push(id); }
        Ok(())
    }
    pub fn valid(&self) -> bool { self.root != 0 && self.nodes.contains_key(&self.root) }
}

#[derive(Clone, Debug)]
pub struct Event { pub kind: u32, pub id: u32, pub value: u32, pub text: String }
pub struct State {
    pub tree: Arc<Tree>,
    pub edits: HashMap<u32, rune::TextEdit>,
    pub combos: HashMap<u32, rune_kit::ComboboxState>,
    pub events: RefCell<Vec<Event>>,
}
impl State {
    pub fn new(tree: Arc<Tree>) -> Self {
        let mut s = Self { tree: tree.clone(), edits: HashMap::new(), combos: HashMap::new(), events: RefCell::new(vec![]) };
        s.install(tree);
        s
    }
    pub fn install(&mut self, tree: Arc<Tree>) {
        self.edits.retain(|id, _| tree.nodes.get(id).is_some_and(|n| n.kind == 5));
        for n in tree.nodes.values().filter(|n| n.kind == 5) {
            let edit = self.edits.entry(n.id).or_insert_with(|| rune::TextEdit::new(n.text.clone()));
            if edit.text != n.text { edit.replace_text(n.text.clone()); }
        }
        self.combos.retain(|id, _| tree.nodes.get(id).is_some_and(|n| n.kind == 9));
        for n in tree.nodes.values().filter(|n| n.kind == 9) { self.combos.entry(n.id).or_default(); }
        self.tree = tree;
    }
    pub fn emit(&self, kind: u32, id: u32, value: u32, text: String) {
        self.events.borrow_mut().push(Event { kind, id, value, text });
    }
}
