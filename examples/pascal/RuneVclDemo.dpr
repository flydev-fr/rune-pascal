// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
program RuneVclDemoApp;
{$APPTYPE CONSOLE}
uses
  {$I mormot.uses.inc}
  SysUtils, Math, Generics.Collections, Forms, RuneVclDemo in 'RuneVclDemo.pas';
var Form:TRuneDemoForm; Mode:string;
begin
  try
    SetExceptionMask([exInvalidOp,exDenormalized,exZeroDivide,exOverflow,exUnderflow,exPrecision]);
    Mode:=ParamStr(1);
    if Pos('standalone',Mode)>0 then StandaloneBench(Pos('shader',Mode)>0,Pos('idle',Mode)>0)
    else begin
      Application.Initialize; Application.MainFormOnTaskbar:=True;
      Application.CreateForm(TRuneDemoForm,Form);
      try Application.Run; finally FreeAndNil(Form); end;
    end;
  except on E:Exception do begin Writeln('ERROR: ',E.ClassName,': ',E.Message); Halt(1); end; end;
end.
