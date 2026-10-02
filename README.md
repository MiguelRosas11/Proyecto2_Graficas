# Santuario de azalea

Diorama en Rust con raytracing en CPU, sin crates externos. Una pequeña isla
con azalea, enredaderas de bayas luminosas y dos antorchas emerge de una laguna.
El borde está al nivel del agua y el árbol un bloque por encima. Rocas bajas con minerales clásicos
aportan acentos de color. El fondo es negro, sin minerales flotantes ni paredes.

## Video del diorama

[Ver el video del santuario de azalea en YouTube](https://youtu.be/UlH-7Wrn1mE)

## Ejecutar

```powershell
cargo run --release
```

Para preparar los recursos en un clon nuevo:

```powershell
powershell -ExecutionPolicy Bypass -File iniciar.ps1
```

La ventana requiere Windows de 64 bits y Rust con el enlazador de C++ de Windows.
El importador usa .NET para extraer imágenes del cliente local de Minecraft o
de recursos oficiales descargados y verificados por SHA-1. El renderizador
solo usa Rust estándar y las API de Windows. Después de importar funciona sin red.
Los recursos de Mojang / Microsoft se excluyen de Git.

## Controles

| Tecla | Acción |
| --- | --- |
| 1 | Órbita general del santuario |
| 2 | Azalea y enredaderas |
| 3 | Agua y reflejos |
| Q / E | Girar |
| W / S | Acercar / alejar manteniendo pulsada la tecla |
| Flechas arriba / abajo | Elevar / bajar la cámara |
| Clic y mouse | Capturar el mouse y controlar la órbita |
| Esc | Liberar el mouse |
| Espacio | Alternar giro automático |
| R | Restaurar cámara |

La rueda no controla el zoom. Las vistas cercanas tienen desplazamientos
limitados; la vista general permite una vuelta completa, con radio de 26 a 34
bloques. No hay menús, vuelo libre ni destrucción de bloques.

## Escena y calidad

- 5,911 bloques con la semilla predeterminada. Sin mineshaft ni minerales lejanos.
- Rocas bajas huecas, terreno de poco espesor y fondo vacío.
- Azalea normal y florecida, helechos y enredaderas sujetas a geometría existente.
- Las bayas de cada cadena comparten fuentes de iluminación para limitar el costo.
- Minerales de piedra gris del paquete oficial Programmer Art; píxeles originales
  de 16 × 16, sin tintes. Sus fuentes ahora pueden iluminar su propia superficie.
- Skybox de seis caras negras, sin sol ni luz ambiental global.
- Sombras por rayos hacia luces locales, resplandor y mapeo de tonos.
- Agua con reflexión primaria, refracción, Fresnel, absorción y ondas suaves.

Mientras se mueve la cámara, la resolución interna predeterminada tiene 640
píxeles de ancho. Después de 350 ms quieta aumenta a 1920 (según la proporción de
la ventana) y acumula 64 muestras para suavizar los bordes. El tiempo se congela
para no mezclar ondas en posiciones diferentes. Al terminar se reutiliza la
imagen y se deja de calcular rayos hasta volver a interactuar. La barra de título
muestra el progreso. El refinamiento tarda varios segundos y puede interrumpirse
moviendo la cámara; no equivale a renderizar esa calidad a 60 FPS.

La iluminación difusa estática usa una caché de 4 × 4 muestras por cara. Las
sombras usan tres muestras por luz; los reflejos y brillos dependen de la vista.
El agua conserva hasta dos niveles de transmisión, pero omite reflexiones
secundarias recursivas. Son aproximaciones; no hay iluminación global completa.
Las luces emisivas de minerales son una interpretación propia, no Minecraft vanilla.

## Rendimiento y capturas

Medición histórica anterior a los cambios de orillas, 480 × 270, release, 30 frames con caché
preparada, sin presentación en ventana: orbital 52 FPS, azalea 41 FPS y agua
28 FPS. Son resultados del equipo y encuadres probados, no garantías. Los
primeros frames pueden ser más lentos y el agua sigue siendo costosa.

El recorrido salta regiones vacías de 8 × 8 × 8. Se emplean hilos estándar,
selección de luces cercanas y caché estática. La cuadrícula de consulta incluye
aire, que no corresponde a cubos dibujados. No se usa la GPU para los rayos.

```powershell
cargo run --release -- --width 320
cargo run --release -- --view azalea
cargo run --release -- --benchmark --view lagoon
cargo run --release -- --snapshot captures/santuario.bmp --width 1440 --samples 32
```

`--width` controla el ancho durante movimiento; al detenerse se triplica, con
máximo de 1920 y alto máximo de 1080. Las capturas usan exactamente el ancho
solicitado. `--samples` acepta 1–64 muestras para capturas. `--angle` fija el
azimut inicial; `--view` admite `overview`, `azalea` y `lagoon`.

## Organización y verificación

| Archivo | Responsabilidad |
| --- | --- |
| `src/world/generation.rs` | Composición y pruebas de escena |
| `src/world/rockwork.rs` | Terreno, laguna, rocas y minerales |
| `src/world/garden.rs` | Azalea, plantas y antorchas |
| `src/world/shore.rs` | Vegetación y luces de las orillas |
| `src/profiling.rs` | Comparaciones controladas de rendimiento |
| `src/camera.rs` | Vistas, límites y zoom |
| `src/material.rs`, `src/texture.rs` | Propiedades ópticas y texturas |
| `src/lighting.rs` | Fuentes, sombras y caché |
| `src/ray.rs`, `src/optics.rs` | Intersecciones y óptica |
| `src/shading.rs`, `src/render.rs` | Color, rayos secundarios y antialiasing |
| `src/postprocess.rs` | Resplandor y mapeo de tonos |
| `src/platform/windows.rs` | Ventana y teclado |

```powershell
cargo run --release -- --profile --width 480
cargo test
cargo clippy --all-targets -- -D warnings
cargo run --release -- --smoke-test
cargo run --release -- --orbit-test --width 480
```

22 pruebas cubren óptica, intersecciones, cámara, presupuesto de geometría,
fondo vacío y equivalencia entre recorridos acelerados y celda por celda. Se
conserva una regresión del bloqueo por redondeo al girar. La prueba orbital
renderiza 1,440 frames (dos vueltas) con el agua animada. La prueba de ventana
espera a completar el refinamiento y guarda `captures/window-smoke.bmp`.

Proyecto educativo no afiliado a Mojang / Microsoft.


El informe actualizado de rendimiento y las comparaciones controladas están en [docs/performance.md](docs/performance.md). El render reparte trabajos de cuatro filas entre los hilos y reutiliza sumas del resplandor. La escena actual tiene borde de isla al nivel del agua, centro un bloque arriba, sin camino sobre el lago, con plantas y antorchas en las orillas.


La configuración de mayor calidad usa 640 píxeles durante el movimiento y hasta 1920 × 1080 con 64 muestras al detenerse. Para priorizar fluidez usa "--width 480". La tabla de mediciones a 480 sigue siendo una referencia, no una medición del nuevo valor predeterminado.
