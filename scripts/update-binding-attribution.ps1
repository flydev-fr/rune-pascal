# SPDX-License-Identifier: MIT
# Copyright (c) 2026 flydev-fr.
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
$path=Join-Path $root 'src\pascal\RunePas.pas'
$text=[IO.File]::ReadAllText($path)
if(-not $text.Contains('https://github.com/flydev-fr/chet-cli')){$text=$text.Replace('  Based on Chet by Erik van Bilsen:', '  ChetCLI by flydev-fr: https://github.com/flydev-fr/chet-cli'+[Environment]::NewLine+'  Based on Chet by Erik van Bilsen:')}
if(-not $text.Contains('SPDX-License-Identifier: MIT')){
 $text='// SPDX-License-Identifier: MIT'+[Environment]::NewLine+'// Copyright (c) 2026 flydev-fr.'+[Environment]::NewLine+$text
}
[IO.File]::WriteAllText($path,$text,[Text.UTF8Encoding]::new($true))
