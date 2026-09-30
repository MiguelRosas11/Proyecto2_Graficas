# Texturas originales de Minecraft

Recursos de Mojang / Microsoft, importados localmente y excluidos de Git.

```powershell
powershell -ExecutionPolicy Bypass -File scripts/import-textures.ps1
powershell -ExecutionPolicy Bypass -File scripts/import-textures.ps1 -ClientJar "C:\ruta\version.jar"
```

Se generan 27 imágenes PPM y sus máscaras `.alpha` (un byte por píxel):
pasto superior/lateral, tierra, piedra, tronco lateral/superior, hojas de azalea
florecida (`oak_leaves`), arena, agua, lava, antorcha, tablones, riel, pizarra
profunda, musgo, diamante, redstone, amatista, enredadera de cueva, oro,
esmeralda, lapislázuli, hierro, hojas de azalea, enredadera con bayas, helecho y arcilla.

Se conservan los píxeles originales, con composición del overlay de pasto y
rieles, tintes de pasto/helechos/agua y recorte del cuerpo de antorcha. Las hojas
y plantas conservan transparencia; el agua usa sus parámetros ópticos en Rust.
Las tiras de animación de agua y lava se conservan completas. La emisión de
minerales se calcula mediante máscaras de color sin repintar sus texturas.

El importador usa System.Drawing y ZipFile de .NET. El motor Rust lee PPM y alfa
con su propio código. `source.txt` registra el cliente utilizado. No se incluyen
estos recursos en el repositorio.

Las seis menas (diamante, oro, hierro, esmeralda, lapislázuli y redstone) usan
ahora las texturas de piedra gris del paquete oficial Programmer Art, con los
diseños clásicos de la referencia. Se verificó que los 16 × 16 píxeles RGB de
cada mena coinciden exactamente con el recurso original; no se aplican tintes.
La emisión de color sigue siendo un efecto propio del renderizador.

El importador busca el paquete en los recursos locales y, si falta, descarga el
recurso oficial identificado por SHA-1 `6a02c65e035f539e878c075c06e6c19e45769b8c`,
verificando su contenido. `-ClassicPack` permite proporcionar un ZIP local.
