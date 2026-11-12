# SPDX-License-Identifier: MIT
# Copyright (c) 2026 flydev-fr.
[CmdletBinding()]
param([switch]$Test,[switch]$Examples,[switch]$BridgeOnly,[switch]$SkipRust,[switch]$Offline,[string]$RustTargetDir='',[string]$DelphiRoot='')
$ErrorActionPreference='Stop';$root=$PSScriptRoot
& "$root\scripts\prepare-dependencies.ps1" -Offline:$Offline
if(-not $RustTargetDir){$RustTargetDir=Join-Path $root 'build\cargo'}
$RustTargetDir=[IO.Path]::GetFullPath($RustTargetDir)
$bin=Join-Path $root 'build\bin';New-Item -ItemType Directory -Force -Path $bin|Out-Null
Push-Location $root
try {
 if(-not $SkipRust){
  $cargoArgs=@('build','--locked','-p','rune-pas','--profile','perf','--target-dir',$RustTargetDir)
  if($Offline){$cargoArgs+='--offline'}
  & cargo @cargoArgs;if($LASTEXITCODE -ne 0){throw 'Rust bridge build failed'}
  if($Test){
   $cargoArgs=@('test','--locked','-p','rune-pas','--lib','--target-dir',$RustTargetDir)
   if($Offline){$cargoArgs+='--offline'}
   & cargo @cargoArgs;if($LASTEXITCODE -ne 0){throw 'Rust bridge tests failed'}
  }
 }
 $dll=Join-Path $RustTargetDir 'perf\rune_pas.dll'
 if(-not(Test-Path -LiteralPath $dll)){throw "Optimized DLL missing: $dll"}
 Copy-Item -LiteralPath $dll -Destination "$bin\rune_pas.dll" -Force
 if(Test-Path -LiteralPath ($dll+'.lib')){Copy-Item -LiteralPath ($dll+'.lib') -Destination "$bin\rune_pas.dll.lib" -Force}
 if($Test -and -not $BridgeOnly){
  foreach($name in 'RunePasTests','RuneLabTests','RuneUiTests','RuneVclTests'){
   & "$root\scripts\invoke-delphi.ps1" -Project "$root\tests\pascal\$name.dproj" -DelphiRoot $DelphiRoot
   & "$bin\$name.exe" /noenter
   if($LASTEXITCODE -ne 0){throw "Tests failed: $name"}
  }
 }
 if($Examples -and -not $BridgeOnly){
  foreach($name in 'SettingsPas','LabPas','TodoPas','ShaderButtonsPas','RuneVclDemo'){
   & "$root\scripts\invoke-delphi.ps1" -Project "$root\examples\pascal\$name.dproj" -DelphiRoot $DelphiRoot
  }
 }
 Write-Host "Rune Pascal bridge: $bin\rune_pas.dll"
}finally{Pop-Location}
