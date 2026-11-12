# SPDX-License-Identifier: MIT
# Copyright (c) 2026 flydev-fr.
[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Project,[string]$DelphiRoot='')
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
$projectPath=(Resolve-Path -LiteralPath $Project).Path
if (-not $DelphiRoot) {
 $candidates=@($env:BDS)
 foreach($key in 'HKCU:\Software\Embarcadero\BDS\37.0','HKLM:\Software\WOW6432Node\Embarcadero\BDS\37.0') {
  $entry=Get-ItemProperty -LiteralPath $key -ErrorAction SilentlyContinue
  if($entry){$candidates+=$entry.RootDir}
 }
 $candidates+=Join-Path ${env:ProgramFiles(x86)} 'Embarcadero\Studio\37.0'
 $DelphiRoot=$candidates | Where-Object {$_ -and (Test-Path -LiteralPath (Join-Path $_ 'bin\rsvars.bat'))} | Select-Object -First 1
}
if(-not $DelphiRoot){throw 'Delphi 13 not found. Use -DelphiRoot or BDS.'}
$rsvars=Join-Path $DelphiRoot 'bin\rsvars.bat'
if(-not (Test-Path -LiteralPath $rsvars)){throw "rsvars.bat not found : $rsvars"}
$cache=Join-Path $root 'build\scripts';New-Item -ItemType Directory -Force -Path $cache | Out-Null
$cmd=Join-Path $cache ('delphi-'+[IO.Path]::GetFileNameWithoutExtension($projectPath)+'.cmd')
@('@echo off',('call "'+$rsvars+'" >nul'),'if errorlevel 1 exit /b %errorlevel%',('MSBuild.exe "'+$projectPath+'" /t:Build /p:Config=Release /p:Platform=Win64 /nologo /v:minimal'),'exit /b %errorlevel%') | Set-Content -LiteralPath $cmd -Encoding Default
& $env:ComSpec /d /c $cmd
if($LASTEXITCODE -ne 0){throw "Delphi build failed : $Project ($LASTEXITCODE)"}
