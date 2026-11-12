# SPDX-License-Identifier: MIT
# Copyright (c) 2026 flydev-fr.
[CmdletBinding()]
param([string]$OutputDirectory='')
$ErrorActionPreference='Stop';$root=Split-Path -Parent $PSScriptRoot
$legal=if($OutputDirectory){[IO.Path]::GetFullPath($OutputDirectory)}else{Join-Path $root 'build\legal'};New-Item -ItemType Directory -Force -Path $legal|Out-Null
Copy-Item -LiteralPath "$root\LICENSE.md" -Destination "$legal\PROJECT-LICENSE.md" -Force
Copy-Item -LiteralPath "$root\THIRD_PARTY.md" -Destination "$legal\THIRD_PARTY.md" -Force
Copy-Item -LiteralPath "$root\LICENSE" -Destination "$legal\LICENSE" -Force
Copy-Item -LiteralPath "$root\NOTICE" -Destination "$legal\NOTICE" -Force
Copy-Item -LiteralPath "$root\LICENSES" -Destination $legal -Recurse -Force
$vendor=(Resolve-Path -LiteralPath "$root\vendor").Path
Get-ChildItem -LiteralPath $vendor -Recurse -File | Where-Object {$_.Name -match '(?i)license|licence|copying|notice|^ofl'} | ForEach-Object {
 $rel=$_.FullName.Substring($vendor.Length+1);$destination=Join-Path $legal ('vendor\'+$rel)
 New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination)|Out-Null
 Copy-Item -LiteralPath $_.FullName -Destination $destination -Force
}
Push-Location $root
try {$tree=& cargo tree --offline --locked -p rune-pas --target x86_64-pc-windows-msvc --edges normal,build --prefix none --format '{p}';if($LASTEXITCODE -ne 0){throw 'Rust dependency inventory failed'}}finally{Pop-Location}
$cargoHome=if($env:CARGO_HOME){$env:CARGO_HOME}else{Join-Path $env:USERPROFILE '.cargo'}
$indexes=Get-ChildItem -LiteralPath (Join-Path $cargoHome 'registry\src') -Directory
$seen=[Collections.Generic.HashSet[string]]::new();$lines=[Collections.Generic.List[string]]::new()
foreach($line in $tree){
 if($line -notmatch '^([\w-]+) v([^\s]+)'){continue}
 $name=$Matches[1];$version=$Matches[2];$key="$name-$version"
 if(-not $seen.Add($key)){continue}
 # Path crates are covered by vendor and the project license above.
 if($line -match '\(' -and $line -notmatch '\(\*\)'){continue}
 $source=$indexes|ForEach-Object {Join-Path $_.FullName $key}|Where-Object {Test-Path -LiteralPath $_}|Select-Object -First 1
 if(-not $source){throw "License sources not found : $key"}
 $manifest=Get-Content -LiteralPath (Join-Path $source 'Cargo.toml') -Raw
 $license='unspecified';if($manifest -match '(?m)^license\s*=\s*"([^"]+)"'){$license=$Matches[1]}
 $lines.Add("$name $version | $license")
 $files=Get-ChildItem -LiteralPath $source -File|Where-Object {$_.Name -match '^(?i:LICENSE|LICENCE|COPYING|NOTICE|UNLICENSE|COPYRIGHT)'}
 $destination=Join-Path $legal ('rust\'+$key);New-Item -ItemType Directory -Force -Path $destination|Out-Null
 foreach($file in $files){Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $destination $file.Name) -Force}
 $lines[-1] += ' | https://crates.io/crates/'+$name+'/'+$version
}
@('Rust dependencies selected for the Windows x64 DLL.','Versions : Cargo.lock. Available license texts are in rust/.','')+($lines|Sort-Object)|Set-Content -LiteralPath "$legal\NOTICES.txt" -Encoding utf8
Write-Host "Third-party notices: $($lines.Count) crates, $legal"
