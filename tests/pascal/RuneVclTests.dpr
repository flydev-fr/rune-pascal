// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
program RuneVclTests;
{$APPTYPE CONSOLE}
uses
  {$I mormot.uses.inc}
  SysUtils, Classes, Math, Forms, Controls, Windows, mormot.core.test, RunePas, RuneUi, RuneVcl, RuneVclScene;
type
  TClipboardPanel=class(TRunePanel)
  protected procedure WriteClipboardText(const Text:UnicodeString); override;
  public
    Copied:UnicodeString;
    Copies:Integer;
    procedure PressKey(Value:Word; Shift:TShiftState);
  end;
  THostCase=class(TSynTestCase)
  private FClicks:Integer; procedure BuildScene(Sender:TObject); procedure Clicked(Sender:TObject);
  private procedure BuildCopyScene(Sender:TObject);
  published procedure EmbeddedLifecycle;
    procedure ClipboardSelection;
  end;
  THostSuite=class(TSynTests)
  published procedure VCL;
  end;
procedure TClipboardPanel.WriteClipboardText(const Text:UnicodeString);
begin Copied:=Text; Inc(Copies); end;
procedure TClipboardPanel.PressKey(Value:Word; Shift:TShiftState);
begin KeyDown(Value,Shift); end;
procedure THostCase.BuildCopyScene(Sender:TObject);
var S:RuneUiStyle; Markup:UTF8String;
begin
  TRuneUi.BeginView;
  try
    TRuneUi.Column(0,1); S:=TRuneUi.DefaultStyle; S.flags:=RUNE_UI_FULL_WIDTH or RUNE_UI_FULL_HEIGHT; TRuneUi.Style(1,S);
    TRuneUi.Add(1,2,RUNE_UI_DOCUMENT,''); Markup:=UTF8Encode('<p>Été café 🦀</p><pre><code>  x := 1;'+#10+'  y := 2;</code></pre>');
    TRuneUi.Check(rune_ui_set_document(2,PAnsiChar(Markup),'',''));
    S:=TRuneUi.DefaultStyle; S.font_size:=16; S.flags:=RUNE_UI_FULL_WIDTH; TRuneUi.Style(2,S); TRuneUi.Commit;
  except TRuneUi.AbortView; raise; end;
end;
procedure THostCase.ClipboardSelection;
var Form:TForm; P:TClipboardPanel;
begin
  Form:=TForm.CreateNew(nil);
  try
    Form.ClientWidth:=800; Form.ClientHeight:=500;
    P:=TClipboardPanel.Create(Form); P.Shaders:=False; P.OnBuildView:=BuildCopyScene;
    P.Align:=alClient; P.Parent:=Form; Form.Show; Application.ProcessMessages; P.FrameOnce;
    P.PressKey(Ord('C'),[ssCtrl]); P.FrameOnce; CheckEqual(P.Copies,0);
    P.PressKey(Ord('A'),[ssCtrl]); P.FrameOnce;
    P.PressKey(Ord('C'),[ssCtrl]); P.FrameOnce;
    CheckEqual(P.Copies,1); Check(Pos('Été café 🦀',P.Copied)>0);
    Check(Pos('  x := 1;'+#10+'  y := 2;',P.Copied)>0);
    P.FrameOnce; CheckEqual(P.Copies,1);
    P.PressKey(VK_INSERT,[ssCtrl]); P.FrameOnce; CheckEqual(P.Copies,2);
  finally Form.Free; end;
end;
procedure THostSuite.VCL; begin AddCase(THostCase); end;
procedure THostCase.BuildScene(Sender:TObject); begin BuildBenchScene(TRunePanel(Sender).Shaders); end;
procedure THostCase.Clicked(Sender:TObject); begin Inc(FClicks); end;
procedure THostCase.EmbeddedLifecycle;
var Form:TForm; P,Q:TRunePanel; Old,Current:UInt32; F:RuneHostFrame;
    Worker:TThread; ForeignCode:Int32; I:Integer;
begin
  CheckEqual(SizeOf(F),40);
  Form:=TForm.CreateNew(nil);
  try
    Form.ClientWidth:=1000; Form.ClientHeight:=700;
    P:=TRunePanel.Create(Form); P.Shaders:=False; P.OnBuildView:=BuildScene; P.Parent:=Form;
    P.SetBounds(10,10,875,325); P.OnRuneClick:=Clicked;
    Form.Show; Application.ProcessMessages; P.FrameOnce;
    Old:=P.HostId; Check(Old<>0);
    CheckEqual(rune_host_input(Old,5,0,0,9,0),0); P.FrameOnce;
    CheckEqual(rune_host_input(Old,5,0,0,32,0),0); P.FrameOnce;
    CheckEqual(FClicks,1); CheckEqual(P.LastClickedId,100);
    P.Width:=800; P.FrameOnce; P.Width:=875; P.FrameOnce;
    P.RecreateForTest; P.FrameOnce; Current:=P.HostId;
    Check(Current<>Old); CheckEqual(rune_host_frame(Old,@F),-1);
    CheckEqual(rune_host_resize(Current,0,100,1),-1);
    Worker:=TThread.CreateAnonymousThread(procedure begin ForeignCode:=rune_host_frame(Current,@F); end);
    Worker.FreeOnTerminate:=False; Worker.Start; Worker.WaitFor; Worker.Free;
    CheckEqual(ForeignCode,-1);
    Q:=TRunePanel.Create(Form); Q.Shaders:=False; Q.OnBuildView:=BuildScene; Q.Parent:=Form; Q.Top:=350;
    Q.FrameOnce; Check(Q.HostId<>Current);
    CheckEqual(rune_host_frame(Current,@F),0);
    Old:=Q.HostId; Q.Free; CheckEqual(rune_host_frame(Old,@F),-1);
    P.Shaders:=True; P.FrameOnce; P.Shaders:=False;
    for I:=1 to 5 do P.FrameOnce;
    CheckEqual(P.FrameOnce.flags and 1,0);
    P.Free; CheckEqual(rune_host_frame(Current,@F),-1);
    CheckEqual(rune_host_destroy(Current),-1);
  finally Form.Free; end;
end;
begin
  SetExceptionMask([exInvalidOp,exDenormalized,exZeroDivide,exOverflow,exUnderflow,exPrecision]);
  Application.Initialize;
  THostSuite.RunAsConsole('Rune VCL embedded lifecycle');
end.
