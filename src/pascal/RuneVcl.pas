// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
unit RuneVcl;
interface
{$I mormot.defines.inc}
uses Windows, Messages, SysUtils, Classes, Math, Controls, ExtCtrls, Graphics, AppEvnts, Clipbrd,
  RunePas, RuneUi;
type
  TRuneEvent = procedure(Sender:TObject; Kind,Id,Value:UInt32; const Text:UnicodeString) of object;
  TRunePanel = class(TCustomControl)
  private
    FHost: UInt32;
    FAppEvents: TApplicationEvents;
    FAnimating:Boolean;
    FShaders, FBenchmark: Boolean;
    FFrames: UInt64;
    FOnRuneClick,FOnBuildView: TNotifyEvent;
    FLastId: UInt32;
    FOnRuneEvent:TRuneEvent;
    FSurrogate:Word;
    procedure BuildView;
    procedure ReleaseHost;
    procedure Idle(Sender:TObject; var Done:Boolean);
    procedure SetShaders(Value:Boolean);
    procedure SendInput(Kind:UInt32; X,Y:Single; Value:UInt32=0; Flags:UInt32=$FFFFFFFF);
    procedure WMErase(var M:TWMEraseBkgnd); message WM_ERASEBKGND;
    procedure WMDialog(var M:TWMGetDlgCode); message WM_GETDLGCODE;
    procedure WMFocusLost(var M:TWMKillFocus); message WM_KILLFOCUS;
    procedure CMMouseLeave(var M:TMessage); message CM_MOUSELEAVE;
  protected
    // Override for a custom clipboard backend; default is Windows Unicode text.
    procedure WriteClipboardText(const Text:UnicodeString); virtual;
    procedure CreateWnd; override;
    procedure DestroyWnd; override;
    procedure Paint; override;
    procedure Resize; override;
    procedure ChangeScale(M,D:Integer; IsDpiChange:Boolean); override;
    procedure MouseMove(Shift:TShiftState; X,Y:Integer); override;
    procedure MouseDown(Button:TMouseButton; Shift:TShiftState; X,Y:Integer); override;
    procedure MouseUp(Button:TMouseButton; Shift:TShiftState; X,Y:Integer); override;
    procedure KeyDown(var Key:Word; Shift:TShiftState); override;
    procedure KeyPress(var Key:Char); override;
    function DoMouseWheel(Shift:TShiftState; WheelDelta:Integer; MousePos:TPoint):Boolean; override;
  public
    constructor Create(AOwner:TComponent); override;
    destructor Destroy; override;
    function FrameOnce: RuneHostFrame;
    procedure Rebuild;
    procedure RecreateForTest;
    property BenchmarkMode:Boolean read FBenchmark write FBenchmark;
    property HostId:UInt32 read FHost;
    property FrameCount:UInt64 read FFrames;
    property LastClickedId:UInt32 read FLastId;
  published
    property Align; property Anchors; property TabStop; property TabOrder;
    property Shaders:Boolean read FShaders write SetShaders default True;
    property OnRuneEvent:TRuneEvent read FOnRuneEvent write FOnRuneEvent;
    property OnBuildView:TNotifyEvent read FOnBuildView write FOnBuildView;
    property OnRuneClick:TNotifyEvent read FOnRuneClick write FOnRuneClick;
  end;
implementation
function HostDispatch(Kind,Id,Value:UInt32; const Text:PAnsiChar; User:Pointer):Int32; cdecl;
var P:TRunePanel;
begin
  Result:=0;
  try
    P:=TRunePanel(User);
    if Kind=RUNE_UI_COPY then P.WriteClipboardText(UTF8ToString(Text));
    if Assigned(P.FOnRuneEvent) then P.FOnRuneEvent(P,Kind,Id,Value,UnicodeString(UTF8ToString(Text)));
    if Kind=RUNE_UI_CLICK then begin P.FLastId:=Id; if Assigned(P.FOnRuneClick) then P.FOnRuneClick(P); end;
  except Result:=1; end;
end;
constructor TRunePanel.Create(AOwner:TComponent);
begin
  inherited; Width:=700; Height:=260; TabStop:=True; FShaders:=True;
  ControlStyle:=ControlStyle+[csOpaque];
  FAppEvents:=TApplicationEvents.Create(Self); FAppEvents.OnIdle:=Idle;
end;
destructor TRunePanel.Destroy;
begin ReleaseHost; inherited; end;
// The application commits its tree here. No VCL message loop is owned by Rune.
procedure TRunePanel.BuildView;
begin
  if Assigned(FOnBuildView) then FOnBuildView(Self)
  else begin TRuneUi.BeginView; TRuneUi.Column(0,1); TRuneUi.Commit; end;
end;
procedure TRunePanel.WriteClipboardText(const Text:UnicodeString);
begin Clipboard.AsText:=Text; end;
procedure TRunePanel.CreateWnd;
var Mask:TFPUExceptionMask;
begin
  inherited;
  if csDesigning in ComponentState then Exit;
  BuildView;
  Mask:=GetExceptionMask;
  SetExceptionMask([exInvalidOp,exDenormalized,exZeroDivide,exOverflow,exUnderflow,exPrecision]);
  try TRuneUi.Check(rune_host_create(NativeUInt(Handle),ClientWidth,ClientHeight,CurrentPPI/96,@HostDispatch,Self,@FHost));
  finally SetExceptionMask(Mask); end;
  Invalidate;
end;
procedure TRunePanel.ReleaseHost;
var Mask:TFPUExceptionMask;
begin
  FAnimating:=False;
  if FHost=0 then Exit;
  Mask:=GetExceptionMask;
  SetExceptionMask([exInvalidOp,exDenormalized,exZeroDivide,exOverflow,exUnderflow,exPrecision]);
  try rune_host_destroy(FHost); FHost:=0; finally SetExceptionMask(Mask); end;
end;
procedure TRunePanel.DestroyWnd;
begin ReleaseHost; inherited; end;
procedure TRunePanel.WMErase(var M:TWMEraseBkgnd); begin M.Result:=1; end;
procedure TRunePanel.WMDialog(var M:TWMGetDlgCode); begin M.Result:=DLGC_WANTARROWS or DLGC_WANTTAB or DLGC_WANTCHARS; end;
procedure TRunePanel.WMFocusLost(var M:TWMKillFocus); begin inherited; SendInput(6,0,0); end;
procedure TRunePanel.CMMouseLeave(var M:TMessage); begin inherited; SendInput(4,0,0); end;
procedure TRunePanel.Paint;
begin
  if csDesigning in ComponentState then begin Canvas.Brush.Color:=clNavy; Canvas.FillRect(ClientRect); Canvas.TextOut(12,12,'Rune GPU surface'); Exit; end;
  if not IsWindowVisible(Handle) then Exit;
  if not FBenchmark then FrameOnce;
end;
function TRunePanel.FrameOnce:RuneHostFrame;
var Mask:TFPUExceptionMask; Code:Int32;
begin
  FillChar(Result,SizeOf(Result),0);
  if (FHost=0) or (ClientWidth=0) or (ClientHeight=0) then Exit;
  Mask:=GetExceptionMask;
  SetExceptionMask([exInvalidOp,exDenormalized,exZeroDivide,exOverflow,exUnderflow,exPrecision]);
  try Code:=rune_host_frame(FHost,@Result); finally SetExceptionMask(Mask); end;
  if Code<>0 then begin FAnimating:=False; TRuneUi.Check(Code); end;
  Inc(FFrames);
  FAnimating:=not FBenchmark and (Result.flags and 1<>0);
end;
// Continue VCL idle processing only while Rune animates. FIFO presentation
// supplies backpressure; a static panel leaves the normal WaitMessage path alone.
procedure TRunePanel.Idle(Sender:TObject; var Done:Boolean);
begin
  if FBenchmark or not FAnimating or not HandleAllocated then Exit;
  if not IsWindowVisible(Handle) or IsIconic(GetAncestor(Handle,GA_ROOT)) then
  begin FAnimating:=False; Exit; end;
  FrameOnce;
  if FAnimating then Done:=False;
end;
procedure TRunePanel.Resize;
begin
  inherited;
  if (FHost<>0) and (ClientWidth>0) and (ClientHeight>0) then
  begin TRuneUi.Check(rune_host_resize(FHost,ClientWidth,ClientHeight,CurrentPPI/96)); Invalidate; end;
end;
procedure TRunePanel.ChangeScale(M,D:Integer; IsDpiChange:Boolean);
begin inherited; Resize; end;
procedure TRunePanel.Rebuild;
begin
  if FHost=0 then Exit;
  BuildView; TRuneUi.Check(rune_host_commit(FHost)); Invalidate;
end;
procedure TRunePanel.SetShaders(Value:Boolean);
begin if Value=FShaders then Exit; FShaders:=Value; Rebuild; end;
procedure TRunePanel.SendInput(Kind:UInt32; X,Y:Single; Value,Flags:UInt32);
begin
  if FHost=0 then Exit;
  if Flags=$FFFFFFFF then begin
    Flags:=0; if GetKeyState(VK_SHIFT)<0 then Flags:=Flags or 1; if GetKeyState(VK_CONTROL)<0 then Flags:=Flags or 2;
  end;
  TRuneUi.Check(rune_host_input(FHost,Kind,X,Y,Value,Flags)); Invalidate;
end;
procedure TRunePanel.MouseMove(Shift:TShiftState; X,Y:Integer); begin inherited; SendInput(1,X,Y); end;
procedure TRunePanel.MouseDown(Button:TMouseButton; Shift:TShiftState; X,Y:Integer);
begin inherited; if Button=mbLeft then begin SetFocus; MouseCapture:=True; SendInput(2,X,Y); end; end;
procedure TRunePanel.MouseUp(Button:TMouseButton; Shift:TShiftState; X,Y:Integer);
begin inherited; if Button=mbLeft then begin SendInput(3,X,Y); MouseCapture:=False; end; end;
procedure TRunePanel.KeyDown(var Key:Word; Shift:TShiftState);
var Flags:UInt32;
begin
  inherited;
  Flags:=0; if ssShift in Shift then Flags:=Flags or 1; if ssCtrl in Shift then Flags:=Flags or 2;
  if (ssCtrl in Shift) and (Key in [Ord('A'),Ord('C')]) then begin SendInput(5,0,0,Key,Flags); Key:=0; Exit; end;
  if (ssCtrl in Shift) and (Key=VK_INSERT) then begin SendInput(5,0,0,Ord('C'),Flags); Key:=0; Exit; end;
  if Key in [VK_TAB,VK_RETURN,VK_ESCAPE,VK_LEFT,VK_RIGHT,VK_UP,VK_DOWN,VK_BACK,VK_DELETE,VK_HOME,VK_END,VK_PRIOR,VK_NEXT] then begin SendInput(5,0,0,Key,Flags); Key:=0; end;
end;
procedure TRunePanel.KeyPress(var Key:Char);
var Code:UInt32;
begin
  inherited;
  if Ord(Key)<32 then Exit;
  Code:=Ord(Key);
  if (Code>=$D800) and (Code<=$DBFF) then begin FSurrogate:=Code; Key:=#0; Exit; end;
  if (Code>=$DC00) and (Code<=$DFFF) and (FSurrogate<>0) then Code:=$10000+((FSurrogate-$D800) shl 10)+(Code-$DC00);
  FSurrogate:=0; SendInput(8,0,0,Code); Key:=#0;
end;
function TRunePanel.DoMouseWheel(Shift:TShiftState; WheelDelta:Integer; MousePos:TPoint):Boolean;
var P:TPoint;
begin
  P:=ScreenToClient(MousePos); SendInput(7,P.X,P.Y,UInt32(Int32(WheelDelta div 2)));
  Result:=True;
end;
procedure TRunePanel.RecreateForTest; begin RecreateWnd; end;
end.
