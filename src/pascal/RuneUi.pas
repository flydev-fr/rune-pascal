// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
unit RuneUi;
{$IFDEF FPC}{$MODE DELPHI}{$ENDIF}
interface
uses SysUtils, Math, RunePas;
type
  { Small optional facade. RunePas.pas remains fully generated. }
  TRuneUi = class
  public
    class procedure Check(Code: Int32); static;
    class procedure BeginView; static;
    class procedure AbortView; static;
    class procedure Commit; static;
    class function DefaultStyle: RuneUiStyle; static;
    class function DefaultEffect(Kind: UInt32): RuneUiEffect; static;
    class procedure Effect(Id: UInt32; const Value: RuneUiEffect); static;
    class procedure Style(Id: UInt32; const Value: RuneUiStyle); static;
    class procedure Add(Parent, Id, Kind: UInt32; const Value: UnicodeString); static;
    class procedure Column(Parent, Id: UInt32); static;
    class procedure Row(Parent, Id: UInt32); static;
    class procedure Text(Parent, Id: UInt32; const Value: UnicodeString); static;
    class procedure Button(Parent, Id: UInt32; const Value: UnicodeString); static;
    class procedure TextInput(Parent, Id: UInt32; const Value: UnicodeString); static;
    class procedure CheckBox(Parent, Id: UInt32; const Value: UnicodeString; Checked: Boolean); static;
    class procedure Run(const Title: UnicodeString; Width, Height: UInt32;
      Callback: RuneUiCallback; UserData: Pointer); static;
  end;
implementation
class procedure TRuneUi.Check(Code: Int32);
var Buffer: array[0..511] of AnsiChar;
begin
  if Code = RUNE_PAS_OK then Exit;
  rune_pas_last_error(@Buffer[0], SizeOf(Buffer));
  raise Exception.Create(string(UTF8Decode(AnsiString(PAnsiChar(@Buffer[0])))));
end;
class procedure TRuneUi.BeginView; begin Check(rune_ui_begin); end;
class procedure TRuneUi.AbortView; begin Check(rune_ui_abort); end;
class procedure TRuneUi.Commit; begin Check(rune_ui_commit); end;
class function TRuneUi.DefaultStyle: RuneUiStyle;
begin Check(rune_ui_style_default(@Result)); end;
class function TRuneUi.DefaultEffect(Kind: UInt32): RuneUiEffect;
begin Check(rune_ui_effect_default(Kind, @Result)); end;
class procedure TRuneUi.Effect(Id: UInt32; const Value: RuneUiEffect);
begin Check(rune_ui_set_effect(Id, @Value)); end;
class procedure TRuneUi.Style(Id: UInt32; const Value: RuneUiStyle);
begin Check(rune_ui_set_style(Id, @Value)); end;
class procedure TRuneUi.Add(Parent, Id, Kind: UInt32; const Value: UnicodeString);
var Utf8: UTF8String;
begin Utf8 := UTF8Encode(Value); Check(rune_ui_add(Parent, Id, Kind, PAnsiChar(Utf8))); end;
class procedure TRuneUi.Column(Parent, Id: UInt32); begin Add(Parent, Id, RUNE_UI_COLUMN, ''); end;
class procedure TRuneUi.Row(Parent, Id: UInt32); begin Add(Parent, Id, RUNE_UI_ROW, ''); end;
class procedure TRuneUi.Text(Parent, Id: UInt32; const Value: UnicodeString); begin Add(Parent, Id, RUNE_UI_TEXT, Value); end;
class procedure TRuneUi.Button(Parent, Id: UInt32; const Value: UnicodeString); begin Add(Parent, Id, RUNE_UI_BUTTON, Value); end;
class procedure TRuneUi.TextInput(Parent, Id: UInt32; const Value: UnicodeString); begin Add(Parent, Id, RUNE_UI_INPUT, Value); end;
class procedure TRuneUi.CheckBox(Parent, Id: UInt32; const Value: UnicodeString; Checked: Boolean);
begin Add(Parent, Id, RUNE_UI_CHECKBOX, Value); Check(rune_ui_set_checked(Id, Ord(Checked))); end;
class procedure TRuneUi.Run(const Title: UnicodeString; Width, Height: UInt32;
  Callback: RuneUiCallback; UserData: Pointer);
var Mask: TFPUExceptionMask; Utf8: UTF8String; Code: Int32;
begin
  Utf8 := UTF8Encode(Title);
  Mask := GetExceptionMask;
  SetExceptionMask([exInvalidOp, exDenormalized, exZeroDivide, exOverflow, exUnderflow, exPrecision]);
  try Code := rune_ui_run(PAnsiChar(Utf8), Width, Height, Callback, UserData);
  finally SetExceptionMask(Mask); end;
  Check(Code);
end;
end.
