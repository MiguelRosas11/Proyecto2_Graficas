$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    $names = 'grass_top','grass_side','dirt','stone','oak_log','oak_log_top','oak_leaves','sand','water','lava','torch','oak_planks','rail','deepslate','moss','diamond','redstone','amethyst','vine','gold','emerald','lapis','iron','azalea','berries','fern','clay'
    $missing = $names | Where-Object { -not (Test-Path -LiteralPath "assets/textures/$_.ppm") -or -not (Test-Path -LiteralPath "assets/textures/$_.alpha") }
    $classic = (Test-Path -LiteralPath "assets/textures/source.txt") -and ((Get-Content "assets/textures/source.txt" -Raw) -match 'classic stone ores v1')
    if ($missing -or -not $classic) { & "$PSScriptRoot/scripts/import-textures.ps1" }
    & cargo run --release -- @args
    if ($LASTEXITCODE -ne 0) { throw "El programa termino con codigo $LASTEXITCODE" }
} finally { Pop-Location }
