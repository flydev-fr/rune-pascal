// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
unit RuneVclScene;
interface
{$I mormot.defines.inc}
uses SysUtils, RunePas, RuneUi;
procedure BuildBenchScene(Shaders: Boolean);
implementation
procedure BuildBenchScene(Shaders: Boolean);
const Titles: array[0..2] of UnicodeString = ('Deploy', 'Confirm', 'Continue');
      Kinds: array[0..2] of UInt32 = (RUNE_FX_GLOW, RUNE_FX_ELECTRIC, RUNE_FX_METAL);
var S: RuneUiStyle; F: RuneUiEffect; I: Integer;
begin
  TRuneUi.BeginView;
  try
    TRuneUi.Column(0,1);
    S:=TRuneUi.DefaultStyle; S.flags:=RUNE_UI_FULL_WIDTH or RUNE_UI_FULL_HEIGHT;
    S.padding:=24; S.gap:=20; S.background_rgba:=$111722FF; TRuneUi.Style(1,S);
    TRuneUi.Text(1,2,'Three buttons, one GPU surface');
    S:=TRuneUi.DefaultStyle; S.font_size:=22; S.font_weight:=600; TRuneUi.Style(2,S);
    TRuneUi.Row(1,3); S:=TRuneUi.DefaultStyle; S.gap:=16; S.height:=56; TRuneUi.Style(3,S);
    for I:=0 to 2 do
    begin
      TRuneUi.Button(3,100+UInt32(I),Titles[I]);
      S:=TRuneUi.DefaultStyle; S.width:=190; S.height:=56; S.radius:=10;
      S.background_rgba:=$263C57FF; S.font_size:=15; S.font_weight:=600; TRuneUi.Style(100+UInt32(I),S);
      if Shaders then begin
        F:=TRuneUi.DefaultEffect(Kinds[I]); F.color_rgba:=$8CBFFFFF;
        if I=0 then F.intensity:=0.6;
        if I=2 then F.color_rgba:=$DCE9FFFF;
        TRuneUi.Effect(100+UInt32(I),F);
      end;
    end;
    TRuneUi.Text(1,4,'VCL keeps its message loop and standard controls.');
    S:=TRuneUi.DefaultStyle; S.foreground_rgba:=$A9B8CEFF; S.font_size:=13; TRuneUi.Style(4,S);
    TRuneUi.Commit;
  except rune_ui_abort; raise; end;
end;
end.
