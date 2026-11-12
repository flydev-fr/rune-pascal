# SPDX-License-Identifier: MIT
# Copyright (c) 2026 flydev-fr.
[CmdletBinding()]
param([string]$Chet='chetcli',[string]$SdkInclude='')
$ErrorActionPreference='Stop';$root=Split-Path -Parent $PSScriptRoot
& $Chet version;if($LASTEXITCODE -ne 0){throw 'ChetCLI unavailable'}
if(-not $SdkInclude){
 $sdk=Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\Include'
 $SdkInclude=Get-ChildItem -LiteralPath $sdk -Directory|Sort-Object Name -Descending|ForEach-Object {Join-Path $_.FullName 'ucrt'}|Where-Object {Test-Path -LiteralPath (Join-Path $_ 'stdint.h')}|Select-Object -First 1
}
$cache=Join-Path $root 'build\bindings';New-Item -ItemType Directory -Force -Path $cache|Out-Null
$argsList=@('generate','--headers',"$root\src\bridge\include",'--out',"$root\src\pascal\RunePas.pas",'--lib-const','LIB_RUNE_PAS','--platform','win64=rune_pas.dll','--char','ansichar','--enum-handling','const','--save-project',"$cache\rune_pas.chet")
if($SdkInclude){$argsList+=@('--clang-arg',('-I'+$SdkInclude))}
# ChetCLI/libclang can also use its bundled standard headers when no SDK is found.
& $Chet @argsList --dry-run
if($LASTEXITCODE -ne 0){throw 'ChetCLI configuration validation failed'}
& $Chet @argsList
if($LASTEXITCODE -ne 0){throw 'ChetCLI generation failed'}

& "$PSScriptRoot\update-binding-attribution.ps1"
