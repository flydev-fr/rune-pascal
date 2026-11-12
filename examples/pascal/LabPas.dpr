// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
program LabPas;
{$APPTYPE CONSOLE}
{$IFDEF FPC}{$MODE DELPHI}{$ENDIF}
uses
  {$I mormot.uses.inc}
  SysUtils,
  Math,
  RunePas;

function Changed(Kind: UInt32; const Config: PRuneLabConfig; User: Pointer): Int32; cdecl;
begin
  Result := 0;
  try
    Writeln('event=', Kind, ' scene=', Config^.scene,
      ' knobs=', Config^.knob_count);
    Flush(Output);
    if (Kind = RUNE_EVENT_READY) and PBoolean(User)^ then Result := 1;
  except
    Result := 1;
  end;
end;

var
  Config: RuneLabConfig;
  Smoke: Boolean;
  Code: Int32;
  I, Arg: Integer;
  Name: array[0..63] of AnsiChar;
  Error: array[0..511] of AnsiChar;
  Mask: TFPUExceptionMask;
begin
  Smoke := FindCmdLineSwitch('smoke');
  if rune_lab_default(@Config) <> RUNE_PAS_OK then Halt(1);
  Config.scene := RUNE_LAB_SCENE_FIRE;
  { Optional scene name, e.g. LabPas.exe lens or LabPas.exe electric. }
  for I := 0 to Integer(rune_lab_scene_count) - 1 do
  begin
    rune_lab_scene_name(I, @Name[0], SizeOf(Name));
    for Arg := 1 to ParamCount do
      if SameText(ParamStr(Arg), string(PAnsiChar(@Name[0]))) then Config.scene := I;
  end;
  Mask := GetExceptionMask;
  SetExceptionMask([exInvalidOp, exDenormalized, exZeroDivide, exOverflow, exUnderflow, exPrecision]);
  try
    Code := rune_lab_run(@Config, @Changed, @Smoke);
  finally
    SetExceptionMask(Mask);
  end;
  if Code <> RUNE_PAS_OK then
  begin
    rune_pas_last_error(@Error[0], SizeOf(Error));
    Writeln('Rune error ', Code, ': ', string(PAnsiChar(@Error[0])));
    Halt(1);
  end;
end.
