// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
unit RuneVclDemo;
interface
{$I mormot.defines.inc}
uses Windows, Messages, SysUtils, Classes, Forms, Controls, StdCtrls, ExtCtrls, Graphics,
  Math, Diagnostics, Generics.Collections, PsAPI, RunePas, RuneUi, RuneVcl, RuneVclScene;
type
  TRuneDemoForm=class(TForm)
  private
    FLabel:TLabel;
    FPanel:TRunePanel;
    FNative:TPanel;
    FToggle:TCheckBox;
    FStart:TTimer;
    FMode:string;
    FIdlePhase:Integer;
    FIdleCpu,FIdleStart,FIdleFrames:Double;
    procedure BuildScene(Sender:TObject);
    procedure Clicked(Sender:TObject);
    procedure Toggle(Sender:TObject);
    procedure Started(Sender:TObject);
    procedure RunBench;
    procedure IdleTick;
  public
    constructor Create(AOwner:TComponent); override;
  end;
procedure StandaloneBench(Shaders,Idle:Boolean);
implementation
function CpuMs:Double;
var C,E,K,U:TFileTime;
begin
  GetProcessTimes(GetCurrentProcess,C,E,K,U);
  Result:=((UInt64(K.dwHighDateTime) shl 32)+K.dwLowDateTime+(UInt64(U.dwHighDateTime) shl 32)+U.dwLowDateTime)/10000;
end;
function PrivateMiB:Double;
var M:PROCESS_MEMORY_COUNTERS_EX;
begin
  FillChar(M,SizeOf(M),0); M.cb:=SizeOf(M);
  GetProcessMemoryInfo(GetCurrentProcess,@M,SizeOf(M)); Result:=M.PrivateUsage/1048576;
end;
function Ms:Double; begin Result:=TStopwatch.GetTimeStamp*1000.0/TStopwatch.Frequency; end;
procedure Line(const Mode:string; Frames:Integer; Median,P95,Elapsed,Cpu:Double; Draws:Double=0; PrivateBytesMiB:Double=-1);
var FS:TFormatSettings;
begin
  FS:=TFormatSettings.Create('en-US');
  if PrivateBytesMiB<0 then PrivateBytesMiB:=PrivateMiB;
  Writeln(Format('RESULT,%s,%d,%.6f,%.6f,%.3f,%.3f,%.3f,%.0f',[Mode,Frames,Median,P95,Elapsed,Cpu,PrivateBytesMiB,Draws],FS)); Flush(Output);
end;
constructor TRuneDemoForm.Create(AOwner:TComponent);
var I:Integer; B:TButton; NativeMode:Boolean;
  function D(N:Integer):Integer; begin Result:=MulDiv(N,Screen.PixelsPerInch,96); end;
begin
  inherited CreateNew(AOwner);
  Caption:='VCL + Rune — prototype'; ClientWidth:=D(748); ClientHeight:=D(390);
  Position:=poScreenCenter; Color:=clBtnFace; Font.Name:='Segoe UI'; Font.Size:=10;
  FMode:=ParamStr(1); NativeMode:=Pos('native',FMode)>0;
  FLabel:=TLabel.Create(Self); FLabel.Parent:=Self; FLabel.SetBounds(D(24),D(16),D(700),D(24));
  FLabel.Caption:='A regular VCL form with three embedded Rune buttons';
  if NativeMode then begin
    FNative:=TPanel.Create(Self); FNative.Parent:=Self; FNative.SetBounds(D(24),D(52),D(700),D(260)); FNative.Caption:='';
    for I:=0 to 2 do begin B:=TButton.Create(Self); B.Parent:=FNative; B.SetBounds(D(24+I*206),D(71),D(190),D(56));
      case I of 0:B.Caption:='Deploy';1:B.Caption:='Confirm';2:B.Caption:='Continue';end; B.OnClick:=Clicked; end;
  end else begin
    FPanel:=TRunePanel.Create(Self); FPanel.Shaders:=Pos('static',FMode)=0;
    FPanel.OnBuildView:=BuildScene; FPanel.Parent:=Self; FPanel.SetBounds(D(24),D(52),D(700),D(260)); FPanel.OnRuneClick:=Clicked;
    FPanel.BenchmarkMode:=Pos('bench',FMode)>0;
  end;
  FToggle:=TCheckBox.Create(Self); FToggle.Parent:=Self; FToggle.SetBounds(D(24),D(330),D(350),D(24));
  FToggle.Caption:='Enable effects (standard VCL control)'; FToggle.Checked:=Assigned(FPanel) and FPanel.Shaders;
  FToggle.Enabled:=Assigned(FPanel); FToggle.OnClick:=Toggle;
  FStart:=TTimer.Create(Self); FStart.Enabled:=False; FStart.Interval:=100; FStart.OnTimer:=Started;
  if FMode<>'' then FStart.Enabled:=True;
end;
procedure TRuneDemoForm.BuildScene(Sender:TObject);
begin BuildBenchScene(TRunePanel(Sender).Shaders); end;
procedure TRuneDemoForm.Clicked(Sender:TObject);
begin
  if Assigned(FPanel) then FLabel.Caption:='Delphi OnClick: button '+IntToStr(FPanel.LastClickedId)
  else FLabel.Caption:='OnClick VCL';
end;
procedure TRuneDemoForm.Toggle(Sender:TObject); begin if Assigned(FPanel) then FPanel.Shaders:=FToggle.Checked; end;
procedure TRuneDemoForm.Started(Sender:TObject);
begin
  FStart.Enabled:=False;
  if Assigned(FPanel) then begin Writeln('SCENE physical=',FPanel.ClientWidth,'x',FPanel.ClientHeight,' dpi=',FPanel.CurrentPPI); Flush(Output); end;
  if Pos('bench',FMode)>0 then begin RunBench; Close; end
  else if Pos('measure-',FMode)=1 then IdleTick
  else if Pos('idle',FMode)>0 then begin FStart.Interval:=3000; FIdlePhase:=0; FStart.Enabled:=True; FMode:='measure-'+FMode; end
  else if FMode='/smoke' then begin
    if Assigned(FPanel) then begin
      FPanel.FrameOnce; FPanel.Width:=MulDiv(650,CurrentPPI,96); FPanel.FrameOnce; FPanel.Width:=MulDiv(700,CurrentPPI,96);
      FPanel.RecreateForTest; FPanel.FrameOnce;
      Writeln('SMOKE: resize + recreate HWND passed');
    end;
    Close;
  end;
end;
procedure TRuneDemoForm.IdleTick;
begin
  if FIdlePhase=0 then begin
    FIdlePhase:=1; FIdleCpu:=CpuMs; FIdleStart:=Ms;
    if Assigned(FPanel) then FIdleFrames:=FPanel.FrameCount else FIdleFrames:=0;
    FStart.Interval:=5000; FStart.Enabled:=True;
  end else begin
    if Assigned(FPanel) then FIdleFrames:=FPanel.FrameCount-FIdleFrames;
    Line(FMode,0,0,0,Ms-FIdleStart,CpuMs-FIdleCpu,FIdleFrames); Close;
  end;
end;
procedure TRuneDemoForm.RunBench;
const Warm=60; Count=180;
var Values:TArray<Double>; I:Integer; Frame:RuneHostFrame; T,Start,Cpu:Double;
begin
  SetLength(Values,Count);
  if Assigned(FPanel) then FPanel.BenchmarkMode:=True;
  for I:=0 to Warm+Count-1 do begin
    if I=Warm then begin Start:=Ms; Cpu:=CpuMs; end;
    if Assigned(FPanel) then begin Frame:=FPanel.FrameOnce; T:=Frame.encode_ms; end
    else begin
      T:=Ms; RedrawWindow(FNative.Handle,nil,0,RDW_INVALIDATE or RDW_UPDATENOW or RDW_ALLCHILDREN); T:=Ms-T;
      Sleep(16); // native paint baseline is paced, not a tight GDI loop
    end;
    if I>=Warm then Values[I-Warm]:=T;
    Application.ProcessMessages;
  end;
  T:=Ms-Start; Cpu:=CpuMs-Cpu;
  TArray.Sort<Double>(Values);
  Line(FMode,Count,Values[Count div 2],Values[Count*95 div 100],T,Cpu,Count);
end;
procedure StandaloneBench(Shaders,Idle:Boolean);
var R:RuneBenchResult; Mask:TFPUExceptionMask; Worker:TThread; IdleCpu,Elapsed,IdlePrivate:Double;
begin
  BuildBenchScene(Shaders);
  if Idle then begin
    IdleCpu:=0; Elapsed:=0; IdlePrivate:=0;
    Worker:=TThread.CreateAnonymousThread(procedure
      var C,T:Double;
      begin Sleep(3000); C:=CpuMs; T:=Ms; Sleep(5000); IdleCpu:=CpuMs-C; Elapsed:=Ms-T; IdlePrivate:=PrivateMiB; rune_ui_quit; end);
    Worker.FreeOnTerminate:=False; Worker.Start;
    TRuneUi.Run('Rune standalone idle',700,260,nil,nil);
    Worker.WaitFor; Worker.Free;
    if Shaders then Line('idle-standalone-shader',0,0,0,Elapsed,IdleCpu,0,IdlePrivate) else Line('idle-standalone-static',0,0,0,Elapsed,IdleCpu,0,IdlePrivate);
  end else begin
    Mask:=GetExceptionMask; SetExceptionMask([exInvalidOp,exDenormalized,exZeroDivide,exOverflow,exUnderflow,exPrecision]);
    try TRuneUi.Check(rune_ui_benchmark(180,@R)); finally SetExceptionMask(Mask); end;
    if Shaders then Line('bench-standalone-shader',R.frames,R.encode_median_ms,R.encode_p95_ms,R.elapsed_ms,0)
    else Line('bench-standalone-static',R.frames,R.encode_median_ms,R.encode_p95_ms,R.elapsed_ms,0);
  end;
end;
end.
