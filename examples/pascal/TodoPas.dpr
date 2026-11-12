// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
program TodoPas;
{$APPTYPE CONSOLE}
{$IFDEF FPC}{$CODEPAGE UTF8}{$ENDIF}
{$IFDEF FPC}{$MODE DELPHI}{$ENDIF}
uses
  {$I mormot.uses.inc}
  SysUtils, RunePas, RuneUi, TodoModel;

var Model: TTodoModel; Smoke: Boolean;

procedure Render;
var S: RuneUiStyle; Fx: RuneUiEffect; I: Integer; Item: TTodoItem; Base: UInt32;
begin
  TRuneUi.BeginView;
  try
    TRuneUi.Column(0, 1);
    S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_FULL_WIDTH or RUNE_UI_FULL_HEIGHT;
    S.padding := 28; S.gap := 14; S.background_rgba := $101318FF; TRuneUi.Style(1, S);
    TRuneUi.Text(1, 2, 'To do');
    S := TRuneUi.DefaultStyle; S.font_size := 30; S.font_weight := 700; TRuneUi.Style(2, S);
    TRuneUi.Text(1, 3, 'Your day, one task at a time.');
    S := TRuneUi.DefaultStyle; S.foreground_rgba := $AAB4C5FF; TRuneUi.Style(3, S);
    TRuneUi.Row(1, 4);
    S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_FULL_WIDTH; S.height := 44; S.gap := 12; TRuneUi.Style(4, S);
    TRuneUi.TextInput(4, 5, Model.Input);
    S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_GROW; S.height := 44; S.padding := 12;
    S.radius := 8; S.background_rgba := $1C222DFF; S.font_size := 15; TRuneUi.Style(5, S);
    TRuneUi.Button(4, 6, '+ Add');
    S := TRuneUi.DefaultStyle; S.width := 120; S.height := 44; S.radius := 8;
    S.background_rgba := $4285EFFF; S.foreground_rgba := $FFFFFFFF; S.font_weight := 600; TRuneUi.Style(6, S);
    Fx := TRuneUi.DefaultEffect(RUNE_FX_GLOW); Fx.color_rgba := $6AA9FFFF;
    Fx.intensity := 0.7; Fx.flags := RUNE_FX_HOVER_ONLY; TRuneUi.Effect(6, Fx);
    TRuneUi.Column(1, 7);
    S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_FULL_WIDTH or RUNE_UI_GROW or RUNE_UI_SCROLL;
    S.gap := 8; TRuneUi.Style(7, S);
    for I := 0 to Model.Count - 1 do
    begin
      Item := Model.Items[I]; Base := 100 + Item.Id * 10;
      TRuneUi.Row(7, Base);
      S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_FULL_WIDTH or RUNE_UI_ANIMATE;
      S.height := 64; S.padding := 12; S.gap := 12; S.radius := 10;
      S.background_rgba := $202632FF; TRuneUi.Style(Base, S);
      TRuneUi.CheckBox(Base, Base + 1, Item.Title, Item.Done);
      S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_GROW; S.gap := 12; S.font_size := 15;
      if Item.Done then S.foreground_rgba := $8994A6FF;
      TRuneUi.Style(Base + 1, S);
      TRuneUi.Button(Base, Base + 2, 'Remove');
      S := TRuneUi.DefaultStyle; S.width := 94; S.height := 32; S.radius := 7;
      S.font_size := 12; S.font_weight := 500;
      S.foreground_rgba := $D7AEB3FF; S.background_rgba := $FFFFFF06;
      TRuneUi.Style(Base + 2, S);
    end;
    TRuneUi.Text(1, 8, UnicodeString(Format('%d task(s)  ·  %d completed', [Model.Count, Model.DoneCount])));
    S := TRuneUi.DefaultStyle; S.foreground_rgba := $AAB4C5FF; TRuneUi.Style(8, S);
    TRuneUi.Text(1, 9, 'Enter to add · Data is kept for this session');
    S := TRuneUi.DefaultStyle; S.font_size := 12; S.foreground_rgba := $7E8AA0FF; TRuneUi.Style(9, S);
    TRuneUi.Commit;
  except
    rune_ui_abort;
    raise;
  end;
end;

function Event(Kind, Id, Value: UInt32; const Text: PAnsiChar; User: Pointer): Int32; cdecl;
var Changed: Boolean;
begin
  Result := 0;
  try
    Changed := False;
    case Kind of
      RUNE_EVENT_READY: if Smoke then Result := 1;
      RUNE_UI_EDIT: if Id = 5 then Model.Input := UTF8Decode(AnsiString(Text));
      RUNE_UI_SUBMIT: if Id = 5 then
        begin Model.Input := UTF8Decode(AnsiString(Text)); Changed := Model.AddDraft; end;
      RUNE_UI_CLICK:
        if Id = 6 then Changed := Model.AddDraft
        else if (Id > 100) and (Id mod 10 = 2) then Changed := Model.Remove((Id - 102) div 10);
      RUNE_UI_CHECK:
        if (Id > 100) and (Id mod 10 = 1) then Changed := Model.SetDone((Id - 101) div 10, Value <> 0);
    end;
    if Changed then Render;
    if Kind <> RUNE_UI_EDIT then
    begin
      Writeln('event=', Kind, ' node=', Id, ' count=', Model.Count, ' done=', Model.DoneCount);
      Flush(Output);
    end;
  except
    on E: Exception do begin Writeln(E.Message); Result := 1; end;
  end;
end;

begin
  Smoke := FindCmdLineSwitch('smoke');
  Model := TTodoModel.Create;
  try
    Model.Input := 'Prepare the day'; Model.AddDraft;
    Model.Input := 'Try a new idea'; Model.AddDraft;
    Model.Input := 'Faire une pause'; Model.AddDraft;
    Model.SetDone(Model.Items[0].Id, True);
    Render;
    TRuneUi.Run('To do — Pascal + Rune', 720, 600, @Event, nil);
  finally Model.Free; end;
end.
