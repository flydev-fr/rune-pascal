// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
program RuneLabTests;
{$APPTYPE CONSOLE}
{$IFDEF FPC}{$MODE DELPHI}{$ENDIF}
uses
  {$I mormot.uses.inc}
  SysUtils,
  Math,
  mormot.core.test,
  RunePas;

type
  TLabContext = record
    InitialScene: UInt32;
    Desired: RuneLabConfig;
    Ready, Changed, Closed: Integer;
    SetResult, QuitResult: Int32;
    Valid: Boolean;
  end;
  PLabContext = ^TLabContext;

function OnLab(Kind: UInt32; const Config: PRuneLabConfig; User: Pointer): Int32; cdecl;
var C: PLabContext; I: Integer;
begin
  Result := 0;
  C := PLabContext(User);
  try
    if (C = nil) or (Config = nil) then begin Result := 1; Exit; end;
    C^.Valid := C^.Valid and (Config^.struct_size = SizeOf(RuneLabConfig))
      and (Config^.knob_count <= RUNE_LAB_MAX_KNOBS);
    case Kind of
      RUNE_EVENT_READY:
        begin
          Inc(C^.Ready);
          C^.Valid := C^.Valid and (Config^.scene = C^.InitialScene);
          { Queue a scene switch and customized shader parameters from Pascal. }
          C^.SetResult := rune_lab_set_config(@C^.Desired);
          if C^.SetResult <> RUNE_PAS_OK then Result := 1;
        end;
      RUNE_EVENT_CHANGED:
        if Config^.scene = C^.Desired.scene then
        begin
          Inc(C^.Changed);
          C^.Valid := C^.Valid and (Config^.knob_count = C^.Desired.knob_count)
            and (Config^.reduce_motion = C^.Desired.reduce_motion);
          for I := 0 to Integer(C^.Desired.knob_count) - 1 do
            C^.Valid := C^.Valid and (Config^.knob_values[I] = C^.Desired.knob_values[I]);
          C^.QuitResult := rune_lab_quit;
          if C^.QuitResult <> RUNE_PAS_OK then Result := 1;
        end;
      RUNE_EVENT_CLOSED: Inc(C^.Closed);
    end;
  except
    if C <> nil then C^.Valid := False;
    Result := 1;
  end;
end;

type
  TLabCase = class(TSynTestCase)
  published
    procedure LayoutMetadataAndValidation;
    procedure LiveShaderConfiguration;
  end;
  TLabSuite = class(TSynTests)
  published
    procedure Lab;
  end;

procedure TLabSuite.Lab;
begin
  AddCase(TLabCase);
end;

procedure TLabCase.LayoutMetadataAndValidation;
var C: RuneLabConfig; K: RuneLabKnobInfo; S, I: Integer;
    Name: array[0..31] of AnsiChar;
begin
  CheckEqual(SizeOf(C), 80);
  CheckEqual(NativeUInt(@C.knob_values) - NativeUInt(@C), 16);
  CheckEqual(SizeOf(K), 116);
  CheckEqual(NativeUInt(@K.name) - NativeUInt(@K), 20);
  CheckEqual(rune_lab_scene_count, 12);
  for S := 0 to Integer(rune_lab_scene_count) - 1 do
  begin
    Check(rune_lab_scene_name(S, nil, 0) > 1);
    Check(rune_lab_scene_name(S, @Name[0], SizeOf(Name)) > 1);
    Check(Name[0] <> #0);
    Check(rune_lab_knob_count(S) <= RUNE_LAB_MAX_KNOBS);
    for I := 0 to Integer(rune_lab_knob_count(S)) - 1 do
    begin
      CheckEqual(rune_lab_knob_info(S, I, @K), RUNE_PAS_OK);
      CheckEqual(K.struct_size, SizeOf(K));
      Check(K.min_value <= K.default_value);
      Check(K.default_value <= K.max_value);
    end;
  end;
  CheckEqual(rune_lab_default(@C), RUNE_PAS_OK);
  CheckEqual(rune_lab_default(nil), RUNE_PAS_INVALID_ARGUMENT);
  CheckEqual(rune_lab_set_config(@C), RUNE_PAS_NOT_RUNNING);
  CheckEqual(rune_lab_quit, RUNE_PAS_NOT_RUNNING);
  C.scene := 99;
  CheckEqual(rune_lab_run(@C, nil, nil), RUNE_PAS_INVALID_ARGUMENT);
  C.scene := RUNE_LAB_SCENE_LAMP;
  C.knob_count := RUNE_LAB_MAX_KNOBS;
  CheckEqual(rune_lab_run(@C, nil, nil), RUNE_PAS_INVALID_ARGUMENT);
  C.knob_count := 0;
  C.struct_size := 4;
  CheckEqual(rune_lab_run(@C, nil, nil), RUNE_PAS_INVALID_ARGUMENT);
end;

procedure TLabCase.LiveShaderConfiguration;
var Config: RuneLabConfig; C: TLabContext; I: Integer;
    K: RuneLabKnobInfo; Mask: TFPUExceptionMask;
begin
  FillChar(C, SizeOf(C), 0);
  C.Valid := True;
  CheckEqual(rune_lab_default(@Config), RUNE_PAS_OK);
  Config.scene := StrToIntDef(GetEnvironmentVariable('RUNE_LAB_TEST_SCENE'), RUNE_LAB_SCENE_FIRE);
  C.InitialScene := Config.scene;
  CheckEqual(rune_lab_default(@C.Desired), RUNE_PAS_OK);
  C.Desired.scene := RUNE_LAB_SCENE_LENS;
  if Config.scene = RUNE_LAB_SCENE_LENS then C.Desired.scene := RUNE_LAB_SCENE_FIRE;
  C.Desired.reduce_motion := 1;
  C.Desired.knob_count := rune_lab_knob_count(C.Desired.scene);
  for I := 0 to Integer(C.Desired.knob_count) - 1 do
  begin
    CheckEqual(rune_lab_knob_info(C.Desired.scene, I, @K), RUNE_PAS_OK);
    C.Desired.knob_values[I] := K.default_value;
    if I = 0 then C.Desired.knob_values[I] := (K.min_value + K.max_value) / 2;
  end;
  Mask := GetExceptionMask;
  SetExceptionMask([exInvalidOp, exDenormalized, exZeroDivide, exOverflow, exUnderflow, exPrecision]);
  try
    CheckEqual(rune_lab_run(@Config, @OnLab, @C), RUNE_PAS_OK);
  finally
    SetExceptionMask(Mask);
  end;
  Check(C.Valid);
  CheckEqual(C.SetResult, RUNE_PAS_OK);
  CheckEqual(C.QuitResult, RUNE_PAS_OK);
  CheckEqual(C.Ready, 1);
  CheckEqual(C.Changed, 1);
  CheckEqual(C.Closed, 1);
  CheckEqual(rune_lab_set_config(@Config), RUNE_PAS_NOT_RUNNING);
  CheckEqual(rune_lab_quit, RUNE_PAS_NOT_RUNNING);
  CheckEqual(rune_lab_run(nil, nil, nil), RUNE_PAS_ALREADY_RUN);
  CheckEqual(rune_settings_run(nil, nil, nil), RUNE_PAS_ALREADY_RUN);
end;

begin
  TLabSuite.RunAsConsole('Rune Lab shader ABI tests');
end.
