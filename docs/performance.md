# Rendimiento: medición controlada y optimización

Medición del 30 de septiembre de 2026, i5-13420H, 480 × 270, release, sin ventana,
una instancia del renderizador y sin compilaciones simultáneas. Cada modo usa
3 frames de preparación y 60 de medición, tiempo de animación fijo. Los tiempos
son suma de medianas de trazado y postprocesado, no percentiles de latencia total.

## Mismo escenario antes y después de optimizar

Esta comparación conserva la escena anterior (5,956 bloques y 16 fuentes),
las cámaras, agua, sombras y resolución. Solo cambian el reparto de trabajo y
el algoritmo del resplandor.

| Vista | Antes, ms | Optimizado, ms |
| --- | ---: | ---: |
| Orbital | 18.84 | 12.62 |
| Azalea | 25.24 | 17.93 |
| Agua | 33.93 | 26.85 |

Se reemplazaron las franjas fijas por trabajos de cuatro filas que los hilos
recogen cuando quedan libres. Esto evita esperar a una franja costosa mientras
otros hilos ya terminaron. No se implementó un grupo permanente de hilos:
se mantienen hilos con ámbito por frame, pero con mejor balance de carga.

El resplandor conserva su filtro separable de nueve muestras y sus parámetros.
Las sumas deslizantes reutilizan resultados en lugar de recalcular nueve vecinos
por píxel. Bajó de unos 4.1 a 1.4 ms a esta resolución. Una prueba compara el
resultado con el algoritmo anterior, con tolerancia de un nivel RGB por redondeo.

## Escenario final con isla baja y orillas plantadas

5,911 bloques y 21 fuentes de luz. Se retiró el camino sobre el agua, se bajó
la isla y se añadieron arbustos, helechos y antorchas en cinco zonas de orilla
que tenían terreno adecuado. Borde en y=9 (igual al agua), centro en y=10.

| Vista | Completo, ms | Agua plana, ms | Sin sombras, ms | Solo intersección primaria, ms |
| --- | ---: | ---: | ---: | ---: |
| Orbital | 14.52 | 9.87 | 11.87 | 8.14 |
| Azalea | 20.05 | 10.01 | 15.08 | 8.07 |
| Agua | 28.06 | 9.97 | 18.16 | 7.74 |

Los modos incompletos son diagnósticos, no ajustes aplicados al programa normal.
Agua plana omite reflexión, refracción y brillos de agua. Sin sombras conserva
luces pero omite sus rayos de visibilidad. Solo intersección primaria conserva
la búsqueda de superficies, incluidos recortes alfa, y omite sombreado. Todos
incluyen el postprocesado. Las diferencias interactúan: no deben sumarse como
porcentajes independientes. La iluminación difusa ya está almacenada en caché.

El agua es el mayor costo removible en su vista cercana. La escena completa
ahora equivale aproximadamente a 69, 50 y 36 FPS respectivamente, sin ventana.
El límite interactivo sigue siendo 60 FPS; giros y superficies nuevas pueden
costar más que un encuadre fijo con caché preparada. No es el tope del hardware.

Reproducir:

```powershell
cargo run --release -- --profile --width 480
```

Se mantiene el refinamiento de 32 muestras y hasta 1440 píxeles de ancho con
la configuración inicial. No se redujeron reflejos, sombras ni muestras para
obtener estas mejoras. Se validaron 1,440 frames de rotación, la ventana con
refinamiento y las pruebas de geometría, rayos y postprocesado.
