param([string]$ClientJar, [string]$ClassicPack)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.IO.Compression.FileSystem
$destination = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../assets/textures'))
[IO.Directory]::CreateDirectory($destination) | Out-Null
$temporaryJar = $null
$temporaryPack = $null
$classicArchive = $null
try {
    if (-not $ClientJar) {
        $localJar = Join-Path $env:APPDATA '.minecraft/versions/1.21.5/1.21.5.jar'
        if (Test-Path -LiteralPath $localJar) { $ClientJar = $localJar }
        else {
            # Pinned official Mojang client, used only to extract eight block textures.
            $temporaryJar = Join-Path ([IO.Path]::GetTempPath()) ('diorama-' + [guid]::NewGuid().ToString() + '.jar')
            Invoke-WebRequest 'https://piston-data.mojang.com/v1/objects/fd19469fed4a4b4c15b2d5133985f0e3e7816a8a/client.jar' -OutFile $temporaryJar
            if ((Get-FileHash -LiteralPath $temporaryJar -Algorithm SHA1).Hash -ne 'FD19469FED4A4B4C15B2D5133985F0E3E7816A8A') { throw 'El archivo descargado no coincide con el cliente oficial.' }
            $ClientJar = $temporaryJar
        }
    }
    # Official Programmer Art pack: the classic stone-backed ores from the reference.
    $classicHash = '6a02c65e035f539e878c075c06e6c19e45769b8c'
    if (-not $ClassicPack) {
        $ClassicPack = Join-Path $env:APPDATA ".minecraft/assets/objects/6a/$classicHash"
        if (-not (Test-Path -LiteralPath $ClassicPack)) {
            $temporaryPack = Join-Path ([IO.Path]::GetTempPath()) ('diorama-classic-' + [guid]::NewGuid().ToString() + '.zip')
            Invoke-WebRequest "https://resources.download.minecraft.net/6a/$classicHash" -OutFile $temporaryPack
            $ClassicPack = $temporaryPack
        }
        if ((Get-FileHash -LiteralPath $ClassicPack -Algorithm SHA1).Hash -ne $classicHash) {
            throw 'El paquete clasico no coincide con el recurso oficial.'
        }
    }
    $classicArchive = [IO.Compression.ZipFile]::OpenRead([IO.Path]::GetFullPath($ClassicPack))
    $archive = [IO.Compression.ZipFile]::OpenRead([IO.Path]::GetFullPath($ClientJar))
    function Read-BlockImage([string]$name) {
        $sourceArchive = if ($name -in @('diamond_ore','redstone_ore','gold_ore','emerald_ore','lapis_ore','iron_ore')) { $classicArchive } else { $archive }
        $entry = $sourceArchive.GetEntry("assets/minecraft/textures/block/$name.png")
        if (-not $entry) { throw "No se encontro la textura $name en $ClientJar" }
        $stream = $entry.Open()
        try { $original = [Drawing.Image]::FromStream($stream); $copy = New-Object Drawing.Bitmap($original); $original.Dispose(); return $copy }
        finally { $stream.Dispose() }
    }
    $mapping = [ordered]@{grass_top='grass_block_top';grass_side='grass_block_side';dirt='dirt';stone='stone';oak_log='oak_log';oak_log_top='oak_log_top';oak_leaves='flowering_azalea_leaves';sand='sand';water='water_still';lava='lava_still';torch='torch';oak_planks='oak_planks';rail='rail';deepslate='deepslate';moss='moss_block';diamond='diamond_ore';redstone='redstone_ore';amethyst='amethyst_block';vine='cave_vines';gold='gold_ore';emerald='emerald_ore';lapis='lapis_ore';iron='iron_ore';azalea='azalea_leaves';berries='cave_vines_lit';fern='fern';clay='clay'}
    $overlay = Read-BlockImage 'grass_block_side_overlay'
    $planks = Read-BlockImage 'oak_planks'
    try {
        foreach ($name in $mapping.Keys) {
            $bitmap = Read-BlockImage $mapping[$name]
            try {
                if ($name -eq 'torch') {
                    $cropped = $bitmap.Clone([Drawing.Rectangle]::new(7,6,2,10),[Drawing.Imaging.PixelFormat]::Format32bppArgb)
                    $bitmap.Dispose(); $bitmap = $cropped
                }
                $buffer = New-Object IO.MemoryStream
$alphaBuffer = New-Object IO.MemoryStream
                $header = [Text.Encoding]::ASCII.GetBytes("P6`n$($bitmap.Width) $($bitmap.Height)`n255`n")
                $buffer.Write($header,0,$header.Length)
                for ($y=0; $y -lt $bitmap.Height; $y++) { for ($x=0; $x -lt $bitmap.Width; $x++) {
                    $color=$bitmap.GetPixel($x,$y)
$alphaBuffer.WriteByte($color.A)
                    $r=[double]$color.R; $g=[double]$color.G; $b=[double]$color.B
                    if ($name -eq 'grass_top' -or $name -eq 'fern') {
                        $r*=0.53; $g*=0.78; $b*=0.29
                        # Opaque foliage for v0.1; retain the original leaf pattern.
                        $alpha=$color.A/255.0; $r=$r*$alpha+28*(1-$alpha); $g=$g*$alpha+55*(1-$alpha); $b=$b*$alpha+18*(1-$alpha)
                    }
                    if ($name -eq 'grass_side') {
                        $top=$overlay.GetPixel($x,$y); $alpha=$top.A/255.0
                        $r=$r*(1-$alpha)+$top.R*0.53*$alpha; $g=$g*(1-$alpha)+$top.G*0.78*$alpha; $b=$b*(1-$alpha)+$top.B*0.29*$alpha
                    }
                    if ($name -eq 'water') { $r*=0.22; $g*=0.64; $b*=0.84 }
                    if ($name -eq 'rail') {
                        $under=$planks.GetPixel($x,$y); $alpha=$color.A/255.0
                        $r=$r*$alpha+$under.R*(1-$alpha); $g=$g*$alpha+$under.G*(1-$alpha); $b=$b*$alpha+$under.B*(1-$alpha)
                    }
                    $buffer.WriteByte([byte]$r);$buffer.WriteByte([byte]$g);$buffer.WriteByte([byte]$b)
                }}
                [IO.File]::WriteAllBytes((Join-Path $destination "$name.ppm"),$buffer.ToArray())
                [IO.File]::WriteAllBytes((Join-Path $destination "$name.alpha"),$alphaBuffer.ToArray())
$alphaBuffer.Dispose(); $buffer.Dispose()
            } finally { $bitmap.Dispose() }
        }
    } finally { $overlay.Dispose(); $planks.Dispose(); $archive.Dispose() }
    [IO.File]::WriteAllText((Join-Path $destination 'source.txt'),"Minecraft client: $ClientJar`nOres: official Programmer Art (classic stone ores v1).`nTextures belong to Mojang / Microsoft. Converted to PPM with grass/foliage tint.`n")
    Write-Host "Texturas listas en $destination"
} finally {
    if ($classicArchive) { $classicArchive.Dispose() }
    if ($temporaryPack -and (Test-Path -LiteralPath $temporaryPack)) { Remove-Item -LiteralPath $temporaryPack }
    if ($temporaryJar -and (Test-Path -LiteralPath $temporaryJar)) { Remove-Item -LiteralPath $temporaryJar }
}
