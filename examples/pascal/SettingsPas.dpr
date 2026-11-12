// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
program SettingsPas;
{$APPTYPE CONSOLE}
{$IFDEF FPC}{$MODE DELPHI}{$ENDIF}
uses
  {$I mormot.uses.inc}
  SysUtils,
  Math,
  RunePas;

function Changed(Kind: UInt32; const State: PRuneSettings; User: Pointer): Int32; cdecl;
begin
  { Never let a Pascal exception cross the DLL boundary. }
  try
    Writeln('event=', Kind, ' theme=', State^.appearance,
      ' accent=', State^.accent, ' speed=', State^.speed:0:2);
    Flush(Output);
    Result := 0;
    if (Kind = RUNE_EVENT_READY) and PBoolean(User)^ then
      Result := 1;
  except
    Result := 1;
  end;
end;

var
  State: RuneSettings;
  Smoke: Boolean;
  Code: Int32;
  SavedMask: TFPUExceptionMask;
  Error: array[0..511] of AnsiChar;
begin
  Smoke := FindCmdLineSwitch('smoke');
  if rune_settings_default(@State) <> RUNE_PAS_OK then
    Halt(1);
  State.appearance := RUNE_APPEARANCE_DARK;
  { Rune/GPU dependencies expect the native C/Rust floating-point environment. }
  SavedMask := GetExceptionMask;
  SetExceptionMask([exInvalidOp, exDenormalized, exZeroDivide, exOverflow, exUnderflow, exPrecision]);
  try
    Code := rune_settings_run(@State, @Changed, @Smoke);
  finally
    SetExceptionMask(SavedMask);
  end;
  if Code <> RUNE_PAS_OK then
  begin
    rune_pas_last_error(@Error[0], SizeOf(Error));
    Writeln('Rune error ', Code, ': ', string(PAnsiChar(@Error[0])));
    Halt(1);
  end;
end.
