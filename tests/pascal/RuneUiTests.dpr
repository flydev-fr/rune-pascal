// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
program RuneUiTests;
{$APPTYPE CONSOLE}
{$IFDEF FPC}{$MODE DELPHI}{$ENDIF}
uses
  {$I mormot.uses.inc}
  SysUtils, mormot.core.test, RunePas, RuneUi, TodoModel;
type
  TContext = record Ready, Committed, Closed: Integer; Valid: Boolean; end;
  PContext = ^TContext;

procedure Build(const Title: UnicodeString; Reduced: Boolean = False);
var S: RuneUiStyle; Fx: RuneUiEffect; I: Integer;
begin
  TRuneUi.BeginView;
  TRuneUi.Column(0, 1);
  S := TRuneUi.DefaultStyle; S.flags := RUNE_UI_FULL_WIDTH or RUNE_UI_FULL_HEIGHT;
  S.padding := 20; S.gap := 4; S.background_rgba := $101318FF; TRuneUi.Style(1, S);
  TRuneUi.Text(1, 2, Title);
  for I := 1 to 8 do
  begin
    TRuneUi.Button(1, 100 + UInt32(I), UnicodeString(IntToStr(I)));
    S := TRuneUi.DefaultStyle; S.width := 200; S.height := 32; S.radius := 8; TRuneUi.Style(100 + UInt32(I), S);
    Fx := TRuneUi.DefaultEffect(I);
    if Reduced then begin Fx.flags := RUNE_FX_REDUCED_MOTION; Fx.speed := 0; end;
    TRuneUi.Effect(100 + UInt32(I), Fx);
  end;
  TRuneUi.Commit;
end;

function Event(Kind, Id, Value: UInt32; const Text: PAnsiChar; User: Pointer): Int32; cdecl;
var C: PContext;
begin
  Result := 0; C := PContext(User);
  try
    case Kind of
      RUNE_EVENT_READY: begin Inc(C^.Ready); C^.Valid := C^.Valid and (Value > 0); Build('Changed from Pascal', True); end;
      RUNE_UI_COMMITTED: begin Inc(C^.Committed); C^.Valid := C^.Valid and (Value > 1); Result := 1; end;
      RUNE_EVENT_CLOSED: Inc(C^.Closed);
    end;
  except C^.Valid := False; Result := 1; end;
end;

type
  TUiCase = class(TSynTestCase)
  published
    procedure PascalModel;
    procedure TransactionValidation;
    procedure NativeTreeUpdate;
  end;
  TUiSuite = class(TSynTests)
  published procedure UI;
  end;

procedure TUiSuite.UI; begin AddCase(TUiCase); end;
procedure TUiCase.PascalModel;
var M: TTodoModel; First, Second: UInt32;
begin
  M := TTodoModel.Create;
  try
    M.Input := '   '; Check(not M.AddDraft); CheckEqual(M.Count, 0);
    M.Input := '  one  '; Check(M.AddDraft); Check(M.Input = '');
    Check(M.Items[0].Title = 'one'); First := M.Items[0].Id;
    M.Input := 'two'; Check(M.AddDraft); Second := M.Items[1].Id;
    Check(First <> Second); Check(M.SetDone(First, True)); CheckEqual(M.DoneCount, 1);
    Check(M.Remove(First)); CheckEqual(M.Count, 1); CheckEqual(M.DoneCount, 0);
    CheckEqual(M.Items[0].Id, Second); Check(not M.Remove(First));
    M.Input := UnicodeString(StringOfChar('x', 513)); Check(not M.AddDraft);
    CheckEqual(M.Count, 1);
  finally M.Free; end;
end;
procedure TUiCase.TransactionValidation;
var S: RuneUiStyle; Fx: RuneUiEffect; I: Integer;
begin
  CheckEqual(SizeOf(Fx), 32);
  CheckEqual(NativeUInt(@Fx.intensity) - NativeUInt(@Fx), 16);
  CheckEqual(rune_ui_effect_default(99, @Fx), -1);
  CheckEqual(rune_ui_effect_default(1, nil), -1);
  CheckEqual(SizeOf(S), 44); CheckEqual(NativeUInt(@S.background_rgba) - NativeUInt(@S), 36);
  CheckEqual(rune_ui_style_default(@S), 0);
  CheckEqual(rune_ui_style_default(nil), -1);
  CheckEqual(rune_ui_quit, RUNE_PAS_NOT_RUNNING);
  CheckEqual(rune_ui_begin, 0); CheckEqual(rune_ui_commit, -1);
  CheckEqual(rune_ui_add(0, 1, RUNE_UI_COLUMN, nil), 0);
  CheckEqual(rune_ui_add(1, 1, RUNE_UI_TEXT, nil), -1);
  CheckEqual(rune_ui_add(999, 2, RUNE_UI_TEXT, nil), -1);
  CheckEqual(rune_ui_add(1, 2, RUNE_UI_TEXT, 'hello'), 0);
  CheckEqual(rune_ui_add(2, 3, RUNE_UI_TEXT, nil), -1);
  CheckEqual(rune_ui_set_checked(2, 1), -1);
  S.struct_size := 4; CheckEqual(rune_ui_set_style(1, @S), -1);
  CheckEqual(rune_ui_effect_default(RUNE_FX_GLOW, @Fx), 0);
  CheckEqual(rune_ui_set_effect(2, @Fx), -1); // labels are not buttons
  CheckEqual(rune_ui_add(1, 3, RUNE_UI_BUTTON, 'shader'), 0);
  for I := 0 to 8 do
  begin
    CheckEqual(rune_ui_effect_default(I, @Fx), 0);
    CheckEqual(rune_ui_set_effect(3, @Fx), 0);
  end;
  CheckEqual(rune_ui_set_choice(2,0),-1);
  CheckEqual(rune_ui_add(1,805,RUNE_UI_COMBOBOX,'English'+#10+'Francais'),0);
  CheckEqual(rune_ui_set_choice(805,1),0);
  CheckEqual(rune_ui_set_choice(805,2),-1);
  CheckEqual(rune_ui_set_choice(805,High(UInt32)),-1);
  CheckEqual(rune_ui_set_choice(805,0),0);
  Fx.intensity := -1; CheckEqual(rune_ui_set_effect(3, @Fx), -1);
  CheckEqual(rune_ui_commit, 0);
  CheckEqual(rune_ui_begin, 0); CheckEqual(rune_ui_abort, 0);
end;
procedure TUiCase.NativeTreeUpdate;
var C: TContext;
begin
  FillChar(C, SizeOf(C), 0); C.Valid := True;
  Build('Built in Pascal');
  TRuneUi.Run('Generic UI test', 480, 400, @Event, @C);
  Check(C.Valid); CheckEqual(C.Ready, 1); CheckEqual(C.Committed, 1); CheckEqual(C.Closed, 1);
  CheckEqual(rune_ui_quit, RUNE_PAS_NOT_RUNNING);
  CheckEqual(rune_ui_run('again', 480, 400, nil, nil), RUNE_PAS_ALREADY_RUN);
end;
begin TUiSuite.RunAsConsole('Pascal-defined UI tests'); end.
