// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
//! Small value palette shared by application chrome and native document flow.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RuneUiTheme {
    pub struct_size: u32,
    pub background: u32,
    pub sidebar: u32,
    pub surface: u32,
    pub foreground: u32,
    pub muted: u32,
    pub border: u32,
    pub code_background: u32,
    pub code_border: u32,
    pub inline_code_background: u32,
    pub quote_background: u32,
    pub quote_border: u32,
    pub table_header: u32,
    pub link: u32,
    pub highlight: u32,
    pub border_width: f32,
    pub panel_radius: f32,
    pub control_radius: f32,
    pub code_radius: f32,
    pub code_padding: f32,
    pub inline_code_padding: f32,
}
impl RuneUiTheme {
    pub fn preset(dark: bool) -> Self {
        let c=if dark { DARK } else { LIGHT };
        Self {struct_size: size_of::<Self>() as u32,
            background:c[0],
            sidebar:c[1],
            surface:c[2],
            foreground:c[3],
            muted:c[4],
            border:c[5],
            code_background:c[6],
            code_border:c[7],
            inline_code_background:c[8],
            quote_background:c[9],
            quote_border:c[10],
            table_header:c[11],
            link:c[12],
            highlight:c[13],
            border_width:1.0,
            panel_radius:14.0,
            control_radius:6.0,
            code_radius:7.0,
            code_padding:16.0,
            inline_code_padding:3.0,
        }
    }
    pub fn valid(&self)->bool {
        self.struct_size==size_of::<Self>() as u32 &&
        [self.border_width,self.panel_radius,self.control_radius,self.code_radius,self.code_padding,self.inline_code_padding].iter().all(|n|n.is_finite() && (0.0..=64.0).contains(n)) && self.border_width<=4.0
    }
}
const DARK:[u32;14]=[421075455, 538976511, 724249599, 3974949631, 2459078655, 909522431, 572662271, 926365183, 808464127, 572662271, 2004118271, 673720319, 2814438911, 1716726527];
const LIGHT:[u32;14]=[4294900735, 4109627391, 3974817023, 690562559, 2088465919, 3789543423, 4126337791, 3789345023, 4008370687, 4143247103, 2862780159, 4042122751, 895458047, 4293173503];

#[cfg(test)]mod tests {
 use super::*;
 #[test]fn palette_abi_and_validation(){
  assert_eq!(size_of::<RuneUiTheme>(),84);
  assert_eq!(std::mem::offset_of!(RuneUiTheme,border_width),60);
  for dark in [false,true] {assert!(RuneUiTheme::preset(dark).valid());}
  let mut t=RuneUiTheme::preset(true);t.code_radius=f32::NAN;assert!(!t.valid());
  t=RuneUiTheme::preset(false);t.border_width=5.0;assert!(!t.valid());
 }
}
