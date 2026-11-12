// SPDX-License-Identifier: MIT
// Copyright (c) 2026 flydev-fr.
unit TodoModel;
{$IFDEF FPC}{$MODE DELPHI}{$ENDIF}
interface
uses SysUtils;
type
  TTodoItem = record
    Id: UInt32;
    Title: UnicodeString;
    Done: Boolean;
  end;
  TTodoModel = class
  private
    FItems: array of TTodoItem;
    FNext: UInt32;
    FInput: UnicodeString;
    function GetCount: Integer;
    function GetDoneCount: Integer;
    function GetItem(Index: Integer): TTodoItem;
  public
    constructor Create;
    function AddDraft: Boolean;
    function Remove(Id: UInt32): Boolean;
    function SetDone(Id: UInt32; Done: Boolean): Boolean;
    property Input: UnicodeString read FInput write FInput;
    property Count: Integer read GetCount;
    property DoneCount: Integer read GetDoneCount;
    property Items[Index: Integer]: TTodoItem read GetItem;
  end;
implementation
constructor TTodoModel.Create; begin inherited Create; FNext := 1; end;
function TTodoModel.GetCount: Integer; begin Result := Length(FItems); end;
function TTodoModel.GetItem(Index: Integer): TTodoItem; begin Result := FItems[Index]; end;
function TTodoModel.GetDoneCount: Integer;
var I: Integer;
begin Result := 0; for I := 0 to High(FItems) do if FItems[I].Done then Inc(Result); end;
function TTodoModel.AddDraft: Boolean;
var Title: UnicodeString; N: Integer;
begin
  Title := Trim(FInput);
  Result := (Title <> '') and (Length(Title) <= 512) and (Count < 500) and (FNext < 400000000);
  if not Result then Exit;
  N := Count; SetLength(FItems, N + 1);
  FItems[N].Id := FNext; FItems[N].Title := Title; FItems[N].Done := False;
  Inc(FNext); FInput := '';
end;
function TTodoModel.Remove(Id: UInt32): Boolean;
var I, J: Integer;
begin
  Result := False;
  for I := 0 to High(FItems) do if FItems[I].Id = Id then
  begin
    for J := I to High(FItems) - 1 do FItems[J] := FItems[J + 1];
    SetLength(FItems, Length(FItems) - 1); Result := True; Exit;
  end;
end;
function TTodoModel.SetDone(Id: UInt32; Done: Boolean): Boolean;
var I: Integer;
begin
  Result := False;
  for I := 0 to High(FItems) do if FItems[I].Id = Id then
  begin FItems[I].Done := Done; Result := True; Exit; end;
end;
end.
