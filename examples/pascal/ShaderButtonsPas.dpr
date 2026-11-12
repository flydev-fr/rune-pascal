// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
program ShaderButtonsPas;
{$APPTYPE CONSOLE}
{$IFDEF FPC}{$CODEPAGE UTF8}{$MODE DELPHI}{$ENDIF}
uses
  {$I mormot.uses.inc}
  SysUtils, RunePas, RuneUi;

const Names: array[1..8] of UnicodeString = (
  'Neon glow', 'Electricity', 'Flame', 'Animated shimmer',
  'Liquid metal', 'Plastic', 'Pulse', 'Glass');
var Reduced, HoverOnly, Smoke: Boolean; Clicks, Last: Integer;

procedure Render;
var S: RuneUiStyle; Fx: RuneUiEffect; I: Integer; Pair, Card, Button: UInt32;
begin
  TRuneUi.BeginView;
  try
    TRuneUi.Column(0, 1);
    S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_FULL_WIDTH or RUNE_UI_FULL_HEIGHT;
    S.background_rgba := $10141DFF; S.padding := 28; S.gap := 14; TRuneUi.Style(1, S);
    TRuneUi.Text(1, 2, 'Buttons and materials');
    S := TRuneUi.DefaultStyle; S.font_size := 28; S.font_weight := 700; TRuneUi.Style(2, S);
    TRuneUi.Text(1, 3, 'Click a button to try its effect.');
    S := TRuneUi.DefaultStyle; S.foreground_rgba := $AAB8CEFF; TRuneUi.Style(3, S);
    TRuneUi.Row(1, 4);
    S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_FULL_WIDTH; S.height := 32; S.gap := 24; TRuneUi.Style(4, S);
    TRuneUi.CheckBox(4, 5, 'Reduced motion', Reduced);
    S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_GROW; TRuneUi.Style(5, S);
    TRuneUi.CheckBox(4, 6, 'Hover-only effects', HoverOnly); TRuneUi.Style(6, S);
    TRuneUi.Column(1, 10);
    S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_GROW or RUNE_UI_SCROLL or RUNE_UI_FULL_WIDTH;
    S.gap := 16; TRuneUi.Style(10, S);
    for I := 1 to 8 do
    begin
      Pair := 20 + UInt32((I - 1) div 2);
      if I mod 2 = 1 then
      begin
        TRuneUi.Row(10, Pair);
        S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_FULL_WIDTH; S.height := 128; S.gap := 20; TRuneUi.Style(Pair, S);
      end;
      Card := 40 + UInt32(I); Button := 100 + UInt32(I);
      TRuneUi.Column(Pair, Card);
      S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_GROW; S.height := 128;
      S.padding := 18; S.gap := 18; S.radius := 14;
      S.background_rgba := $1B2433FF; TRuneUi.Style(Card, S);
      TRuneUi.Text(Card, 60 + UInt32(I), Names[I]);
      S := TRuneUi.DefaultStyle; S.font_size := 12; S.foreground_rgba := $AAB8CEFF; TRuneUi.Style(60 + UInt32(I), S);
      TRuneUi.Button(Card, Button, 'Essayer');
      S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_FULL_WIDTH; S.height := 48; S.radius := 12;
      S.background_rgba := $253C57FF; S.font_size := 15; S.font_weight := 600;
      if I = RUNE_FX_GLASS then S.background_rgba := $00000000;
      TRuneUi.Style(Button, S);
      Fx := TRuneUi.DefaultEffect(I);
      if I = RUNE_FX_GLOW then begin Fx.color_rgba := $64D8FFFF; Fx.intensity := 0.8; end;
      if I = RUNE_FX_ELECTRIC then Fx.color_rgba := $B39BFFFF;
      if I = RUNE_FX_METAL then Fx.color_rgba := $DCE9FFFF;
      if I = RUNE_FX_PLASTIC then Fx.color_rgba := $4385D6FF;
      if I = RUNE_FX_PULSE then Fx.color_rgba := $61E1B6FF;
      if Reduced then Fx.flags := Fx.flags or RUNE_FX_REDUCED_MOTION;
      if HoverOnly then Fx.flags := Fx.flags or RUNE_FX_HOVER_ONLY;
      TRuneUi.Effect(Button, Fx);
    end;
    if Last = 0 then TRuneUi.Text(1, 8, 'Labels stay sharp and buttons remain interactive.')
    else TRuneUi.Text(1, 8, Names[Last] + ' — ' + UnicodeString(IntToStr(Clicks)) + ' clic(s)');
    S := TRuneUi.DefaultStyle; S.foreground_rgba := $B6C5DBFF; TRuneUi.Style(8, S);
    TRuneUi.Commit;
  except rune_ui_abort; raise; end;
end;

function Event(Kind, Id, Value: UInt32; const Text: PAnsiChar; User: Pointer): Int32; cdecl;
begin
  Result := 0;
  try
    case Kind of
      RUNE_EVENT_READY: if Smoke then Result := 1;
      RUNE_UI_CHECK: begin
        if Id = 5 then Reduced := Value <> 0;
        if Id = 6 then HoverOnly := Value <> 0;
        Render;
      end;
      RUNE_UI_CLICK: if (Id >= 101) and (Id <= 108) then
        begin Inc(Clicks); Last := Id - 100; Render; end;
    end;
    Writeln('event=', Kind, ' node=', Id, ' clicks=', Clicks, ' reduced=', Ord(Reduced), ' hover=', Ord(HoverOnly));
    Flush(Output);
  except on E: Exception do begin Writeln(E.Message); Result := 1; end; end;
end;
begin
  Smoke := FindCmdLineSwitch('smoke');
  Reduced := FindCmdLineSwitch('reduced');
  HoverOnly := FindCmdLineSwitch('hover');
  Render;
  TRuneUi.Run('Materials and lighting — Pascal + Rune', 860, 860, @Event, nil);
end.
