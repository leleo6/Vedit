# Vedit - Arquitectura y especificaciones técnicas (v1)

### Arquitectura Básica
```text
vedit/
├── Cargo.toml
│
└── crates/
    │
    ├── vedit-core/
    │   └── src/
    │       ├── lib.rs
    │       │
    │       ├── project/               # Modelo de datos y serialización
    │       │   ├── mod.rs             # Project, ProjectMetadata
    │       │   ├── track.rs           # Track, TrackKind
    │       │   ├── clip.rs            # AudioClip, VideoClip, ImageClip, TextClip
    │       │   └── io.rs              # I/O asíncrono para JSON (Tokio)
    │       │
    │       ├── tools/
    │       │   ├── mod.rs             # Tool trait
    │       │   ├── audio/
    │       │   │   ├── mod.rs
    │       │   │   ├── add_track.rs
    │       │   │   ├── add_clip.rs
    │       │   │   ├── mix.rs
    │       │   │   ├── mute.rs
    │       │   │   ├── normalize.rs
    │       │   │   └── fade.rs
    │       │   ├── video/
    │       │   ├── image/
    │       │   └── text/
    │       │       ├── mod.rs
    │       │       ├── add_track.rs
    │       │       ├── add_clip.rs
    │       │       ├── style.rs
    │       │       └── subtitle.rs
    │       │
    │       ├── render/                # Renderización
    │       │   ├── mod.rs             # RenderJob, RenderOutput
    │       │   ├── audio.rs           # renderiza solo audio
    │       │   ├── video.rs           # renderiza video base
    │       │   ├── text.rs            # renderiza solo texto (previews y filtros)
    │       │   └── compositor.rs      # mezcla todo, output final
    │       │
    │       ├── motion/                # Sistema de animación y regiones
    │       │   ├── mod.rs             # MovementFormula, RenderRegion
    │       │   ├── formula.rs         # Lógica de expresiones FFmpeg
    │       │   ├── presets.rs         # Catálogo de movimientos (dvd, pulse, etc.)
    │       │   └── region.rs          # Tipado de rangos temporales
    │       │
    │       ├── ffmpeg/                # Integración con FFmpeg subyacente
    │       │   ├── command.rs
    │       │   ├── mod.rs
    │       │   └── probe.rs
    │       │
    │       ├── history/               # Lógica de Undo/Redo
    │       │   └── mod.rs
    │       │
    │       ├── cache/                 # Gestión de archivos temporales y proxies
    │       │   └── mod.rs
    │       │
    │       ├── config/                # Configuración global del usuario
    │       │   └── mod.rs             # VeditConfig (encoder, HW accel, paths)
    │       │
    │       └── context/               # Contexto de ejecución para plugins/efectos
    │           └── mod.rs
    │
    ├── vedit-cli/
    │   └── src/
    │       ├── main.rs
    │       └── commands/
    │           ├── mod.rs
    │           ├── project.rs         # new, open, info, timeline
    │           ├── track.rs           # add, remove, list
    │           ├── clip.rs            # add, remove, move
    │           ├── audio.rs           # mix, mute, normalize, fade
    │           ├── video.rs           # add, transform, speed, color, effects, transition
    │           ├── image.rs           # add, move, transform, mode
    │           ├── text.rs            # add, style, position, import-srt
    │           ├── render.rs          # render con opciones (export-frame, text-preview)
    │           └── config.rs          # set, show, check, reset globales
    └── vedit-gui/
        └── src/
            └── main.rs                # placeholder (futuro frontend)
```

### Frameworks y Librerías Base
- **clap (v4)**: Para el parsing del CLI con el feature de derive (muy limpio).
- **indicatif**: Librería estándar de facto para barras de progreso. Indispensable para evitar la apariencia de cuelgues durante procesos largos.
- **tokio**: Motor asíncrono para I/O (lectura/escritura de archivos JSON) y procesamiento concurrente.
- **anyhow / thiserror**: Manejo de errores robusto y descriptivo sin caer en runtime panics.
- **tracing**: Logs estructurados (filtrado por niveles como debug, info, error).
- **serde / serde_json**: Serialización estricta del proyecto.
- **uuid / chrono**: Generación de identificadores de clips y marcas de tiempo del historial.
- **console**: Para la renderización de la línea de tiempo interactiva (timeline) con colores adaptables al ancho de la terminal.

### Seguridad y Arquitectura Reciente
- Migración a operaciones de I/O de archivos utilizando un modelo completamente asíncrono (Tokio) para prevenir el bloqueo del hilo principal.
- Implementación de validaciones estrictas de integridad del proyecto (asegurando que los medios existen y que hay clips en la línea de tiempo) antes de ejecutar tareas intensivas de renderizado.
- Sistema de Undo/Redo acotado en memoria a un stack máximo de 50 operaciones para prevenir _memory leaks_ en proyectos de alta complejidad.
- Dependencia principal de delegación en **FFmpeg**, usando comandos eficientes desde Rust en lugar de bindings poco mantenidos de bibliotecas de terceros.
- Incorporación de un comando `vedit project timeline` para inspección visual y responsiva de los tracks sin necesidad de un entorno gráfico.

### Formatos Soportados (Salida)
- Resolución de Pantalla: `9:16` y `16:9`
- Video: `MP4`, `MKV`, `MOV`
- Solo Audio: `MP3`, `WAV`, `AAC`, `FLAC`, `OGG`

### Formatos Soportados (Entrada)
- Video: `MP4`, `MKV`, `MOV`, `AVI`, `WEBM`
- Audio: `MP3`, `WAV`, `AAC`, `FLAC`, `OGG`, `M4A`
- Imágenes : `PNG`, `JPG`

### Capacidades del Módulo de Audio
- **Gestión de Tracks**
    - Crear track de audio con nombre, volumen base y estado muted.
    - Eliminar y renombrar tracks.
    - Ajustar volumen global del track.
    - Mutear / desmutear track.
    - Reordenar tracks (cambia prioridad en el mix).

- **Gestión de Clips (dentro de un track)**
    - Agregar clip desde archivo fuente con posición definida (timeline_start) y duración manual opcional para omitir ffprobe.
    - Eliminar clip o mover de posición.
    - Recortar clip (ajustar source_start / source_end limitando la duración fuente).
    - Ajustar el volumen individual independiente del track.
    - Dividir clip por la mitad en la posición inicial (split).
    - Función Loop — repetir un clip de audio N veces o hasta copar la duración.

- **Efectos por Clip**
    - Fade In y Fade Out con especificación de duración en segundos.
    - Desactivación de sonido (mute de sub-bloque).
    - Alteración algorítmica de Pitch y Velocidad.

- **Procesamiento por Track**
    - Normalización de volumen del track de audio (Target: loudness, -23 LUFS).
    - Fade in / out paramétrico de master del track.

- **Mezcla Final**
    - Hacer Bounce o mezclar múltiples canales (tracks de audio) directamente a uno.
    - Extracción de audio embebido de los clips de video.
    - Sustitución rápida del audio del video para uso referencial.

- **Render**
    - Render de solo archivo de sonido (only-audio render).
    - Mezclador final en contenedor multimedia (`video.rs` + `audio.rs` -> `compositor.rs`).

### Módulo de Imagen

- **Gestión de tracks**

- Crear track de imagen con nombre y orden de capa
- Eliminar track
- Renombrar track
- Reordenar tracks (prioridad visual, el track 1 va encima del 2)

- **Gestión de clips dentro de un track**

- Agregar imagen al timeline con posición y duración (timeline_start, duration)
- Eliminar clip
- Mover clip en el timeline
- Ajustar duración del clip
- Dividir clip en un punto del timeline

- **Posicionamiento y transformación (Dinámico)**

- Posición en el frame (x, y) — soporta fórmulas FFmpeg (ej. `sin(t)*100`)
- Escala (ancho x alto) — soporta fórmulas FFmpeg (ej. `1.0 + 0.1*sin(t)`)
- Rotación en grados
- Opacidad (0.0 a 1.0)
Recortar imagen (crop antes de colocarla)

- **Modos de uso**

- Overlay — encima del video con posición, escala y opacidad
- Fondo — ocupa todo el frame, reemplaza el video en ese rango de tiempo
- Pantalla completa con duración — imagen estática por X segundos (para slideshows)

- **Efectos por clip**

- Fade in / fade out de opacidad
- Ken Burns effect (zoom + pan lento sobre la imagen) — muy útil para slideshows
- Animación de entrada: slide desde un lado, aparecer desde centro

- **Formatos de entrada soportados**

- jpg, jpeg, png, webp, gif (primer frame), bmp

- **Render**

- Renderizar proyecto con imágenes como parte del compositor (video + audio)
- Renderizar slideshow puro (only image + audio)

### Módulo de Video

- **Gestión de tracks**

- Crear track de video con nombre y orden de capa
- Eliminar track
- Renombrar track
- Reordenar tracks (prioridad visual)
- Mutear track (deshabilitar sin eliminar)

- **Gestión de clips dentro de un track**

- Agregar clip desde archivo fuente con posición en el timeline
- Eliminar clip
- Mover clip en el timeline
- Recortar clip (source_start / source_end)
- Dividir clip en un punto del timeline
- Ajustar duración (stretch sin cambiar velocidad)

- **Transformación (Dinámica)**

- Escala (soporta expresiones matemáticas evaluadas por frame)
- Posición en el frame (x, y) (soporta expresiones matemáticas)
- Rotación en grados
- Flip horizontal / vertical
- Crop (recortar región del frame)

- **Velocidad**

- Cambiar velocidad del clip (0.25x, 0.5x, 2x, etc)
- Reverse (reproducir clip al revés)
- Mantener pitch del audio al cambiar velocidad

- **Corrección de color**

- Brillo / contraste
- Saturación
- Balance de blancos (temperatura de color)
- Curvas RGB básicas
- LUT (aplicar un archivo .cube de color grading)

- **Efectos visuales y filtros**

- Blur (gaussiano)
- Sharpen
- Vignette
- Noise / grain
- Deinterlace (para material de cámara antigua)

- **Transiciones entre clips**

- Cut (corte directo, sin transición)
- Fade in / fade out (a negro o a blanco)
- Cross dissolve (fundido entre dos clips)
- Wipe (barrido horizontal o vertical)

- **Estabilización**

- Estabilización de video (deshake) — FFmpeg tiene vidstabdetect + vidstabtransform

- **Render**

- Renderizar solo video (only video)
- Renderizar video + audio como parte del compositor final
- Exportar frame específico como imagen (screenshot)

### Módulo de Texto / Subtítulos

- **Gestión de tracks**

- Crear track de texto con nombre y orden de capa
- Eliminar track
- Renombrar track
- Reordenar tracks (prioridad visual)

- **Gestión de clips de texto**

- Agregar texto estático al timeline con posición y duración
- Eliminar clip
- Mover clip en el timeline
- Ajustar duración del clip
- Dividir clip en un punto del timeline

- **Estilo**

- Fuente (font family)
- Tamaño de fuente
- Color del texto
- Color de fondo / caja detrás del texto (con opacidad)
- Negrita, cursiva, subrayado
- Alineación (izquierda, centro, derecha)
- Interlineado y espaciado entre letras
- Stroke (borde alrededor del texto con color y grosor)
- Sombra (offset x, y, blur, color)

- **Posicionamiento**

- Posición en el frame (x, y) — coordenadas absolutas o presets (top-center, bottom-center, etc)
- Margen desde los bordes
- Rotación en grados

- **Efectos por clip**

- Fade in / fade out de opacidad
- Animación de entrada: typewriter (letra por letra), slide, fade
- Animación de salida: fade, slide

- **Subtítulos**

- Importar archivo .srt y convertirlo en clips de texto automáticamente
- Importar archivo .vtt
- Exportar clips de texto como .srt
- Quemar subtítulos en el video (hardcode) — no se pueden quitar después
- Subtítulos suaves (softcode) — se incrustan como stream separado en el archivo

- **Render**

- Renderizar texto como parte del compositor final
- Preview de texto sin renderizar todo el proyecto

### Módulo de Movimiento Dinámico (Nuevo)

- **Fórmulas Matemáticas (Integración FFmpeg)**
    - Evaluación por frame (`eval=frame`) para animaciones fluidas.
    - Acceso a variables de tiempo (`t`) y número de frame (`n`).
    - Redondeo automático de dimensiones (`trunc`) para compatibilidad con codecs.

- **Catálogo de Presets**
    - `dvd_bounce`: Rebote diagonal clásico.
    - `pulse_slow/fast`: Efecto de respiración/latido.
    - `drift_left/right`: Traslación lineal constante.
    - `shake`: Vibración aleatoria-determinista mediante armónicos.
    - `orbit`: Movimiento circular alrededor de un centro.

- **Control de Regiones**
    - Definición de `RenderRegion` (inicio y duración).
    - Soporte para renderizado parcial del timeline para feedback rápido.

### Módulo de Render

- **Configuración de salida**

- Resolución (1920x1080, 1280x720, 3840x2160, custom)
- Aspect ratio (16:9, 9:16, 1:1, 4:3, custom)
- FPS (24, 25, 30, 60, custom)
- Codec de video (H.264, H.265/HEVC, VP9, AV1)
- Codec de audio (AAC, MP3, FLAC, OGG)
- Bitrate de video (auto o manual)
- Bitrate de audio (auto o manual)
- Formato de contenedor (mp4, mkv, mov)

- **Modos de render**

- only audio — exporta solo el mix de audio del proyecto
- only video — exporta solo el video sin audio
- only image — exporta slideshow de imágenes
- video + audio — compositor completo, output final
- frame — exporta un frame específico como imagen (jpg, png)

- **Pipeline de render**

- Validar proyecto antes de renderizar (clips faltantes, conflictos de timeline)
- Construir grafo de filtros FFmpeg a partir de los tracks
- Resolver orden de composición (capas de video, imagen, texto)
- Mezclar todos los tracks de audio
- Aplicar efectos en orden correcto
- Manejar archivos temporales intermedios via cache/

- **Progreso y control**

- Progreso en tiempo real (frame actual / total, porcentaje, tiempo estimado)
- Cancelar render en progreso
- Pausar / reanudar render
- Log de errores durante el render

- **Optimización y Control Avanzado**

- **Renderizado por Regiones**: Renderizar solo el rango seleccionado (`--start-time`, `--duration`).
- **Motion Overlays**: Aplicar fórmulas de movimiento global sobre la composición final.
- **Evaluación Dinámica**: Los filtros de escala y posición se recalculan en cada frame.
- **Seguridad de Dimensiones**: Forzado de dimensiones pares para compatibilidad total con `libx264`.

- **Presets de Render**

- Guardar configuración de render como preset reutilizable.
- Listar y previsualizar fórmulas de movimiento disponibles (`render motion list`).

- **Configuración Global**

- Configurar rutas personalizadas de `ffmpeg` y `ffprobe`.
- Definir hardware encoding por defecto (`h264_nvenc`, `h264_vaapi`, `h264_amf`).
- Asignar valores predeterminados para nuevos proyectos (resolución, FPS).
- Controlar el uso de CPU con `max_threads` y limpieza automática del `cache_dir`.
- Subcomando `vedit config check` para verificar la instalación y capacidades.