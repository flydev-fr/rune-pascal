// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
program RunePasTests;
{$APPTYPE CONSOLE}
{$IFDEF FPC}{$MODE DELPHI}{$ENDIF}
uses
  {$I mormot.uses.inc}
  SysUtils,
  Math,
  mormot.core.test,
  RunePas;

type
  TCallbackState = record
    Ready, Closed: Integer;
    Valid: Boolean;
  end;
  PCallbackState = ^TCallbackState;

function OnEvent(Kind: UInt32; const State: PRuneSettings; User: Pointer): Int32; cdecl;
var C: PCallbackState;
begin
  Result := 1;
  { Only POD operations: no exception or reference may escape this callback. }
  C := PCallbackState(User);
  if (C = nil) or (State = nil) then
    Exit;
  C^.Valid := C^.Valid and (State^.struct_size = SizeOf(RuneSettings))
    and (State^.appearance = RUNE_APPEARANCE_DARK) and (State^.accent = 2);
  if Kind = RUNE_EVENT_READY then Inc(C^.Ready);
  if Kind = RUNE_EVENT_CLOSED then Inc(C^.Closed);
end;

type
  TBindingCase = class(TSynTestCase)
  published
    procedure ABIAndErrors;
    procedure NativeWindowCallback;
  end;
  TBindingSuite = class(TSynTests)
  published
    procedure Binding;
  end;

procedure TBindingSuite.Binding;
begin
  AddCase(TBindingCase);
end;

procedure TBindingCase.ABIAndErrors;
var S: RuneSettings; Error: array[0..1] of AnsiChar; Required: UInt32;
begin
  CheckEqual(rune_pas_abi_version, RUNE_PAS_ABI_VERSION_CURRENT);
  CheckEqual(SizeOf(S), 32);
  CheckEqual(NativeUInt(@S.speed) - NativeUInt(@S), 24);
  CheckEqual(rune_settings_default(@S), RUNE_PAS_OK);
  CheckEqual(S.struct_size, 32);
  Check(S.speed = 0.5);
  CheckEqual(rune_settings_default(nil), RUNE_PAS_INVALID_ARGUMENT);
  Required := rune_pas_last_error(nil, 0);
  Check(Required > 1);
  CheckEqual(rune_pas_last_error(@Error[0], SizeOf(Error)), Required);
  Check(Error[1] = #0);
  S.accent := 99;
  CheckEqual(rune_settings_run(@S, nil, nil), RUNE_PAS_INVALID_ARGUMENT);
end;

procedure TBindingCase.NativeWindowCallback;
var S: RuneSettings; C: TCallbackState; SavedMask: TFPUExceptionMask;
begin
  CheckEqual(rune_settings_default(@S), RUNE_PAS_OK);
  S.appearance := RUNE_APPEARANCE_DARK;
  S.accent := 2;
  FillChar(C, SizeOf(C), 0);
  C.Valid := True;
  SavedMask := GetExceptionMask;
  SetExceptionMask([exInvalidOp, exDenormalized, exZeroDivide, exOverflow, exUnderflow, exPrecision]);
  try
    CheckEqual(rune_settings_run(@S, @OnEvent, @C), RUNE_PAS_OK);
  finally
    SetExceptionMask(SavedMask);
  end;
  Check(C.Valid);
  CheckEqual(C.Ready, 1);
  CheckEqual(C.Closed, 1);
  CheckEqual(rune_settings_run(nil, nil, nil), RUNE_PAS_ALREADY_RUN);
end;

begin
  TBindingSuite.RunAsConsole('Rune Pascal ABI tests');
end.
