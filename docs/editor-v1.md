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
    │       │   ├── clip.rs            # AudioClip, VideoClip, ImageClip
    │       │   └── io.rs              # I/O asíncrono para JSON (Tokio)
    │       │
    │       ├── tools/
    │       │   ├── mod.rs             # Tool trait
    │       │   └── audio/
    │       │       ├── mod.rs
    │       │       ├── add_track.rs
    │       │       ├── add_clip.rs
    │       │       ├── mix.rs
    │       │       ├── mute.rs
    │       │       ├── normalize.rs
    │       │       └── fade.rs
    │       │
    │       ├── render/                # Renderización
    │       │   ├── mod.rs             # RenderJob, RenderOutput
    │       │   ├── audio.rs           # renderiza solo audio
    │       │   ├── video.rs           # renderiza solo video
    │       │   └── compositor.rs      # mezcla todo, output final
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
    │       └── context/               # Contexto de ejecución para plugins/efectos
    │           └── mod.rs
    │
    ├── vedit-cli/
    │   └── src/
    │       ├── main.rs
    │       └── commands/
    │           ├── mod.rs
    │           ├── project.rs         # new, open, info
    │           ├── track.rs           # add, remove, list
    │           ├── clip.rs            # add, remove, move
    │           ├── audio.rs           # mix, mute, normalize, fade
    │           └── render.rs          # render con opciones
    │
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

### Seguridad y Arquitectura Reciente
- Migración a operaciones de I/O de archivos utilizando un modelo completamente asíncrono (Tokio) para prevenir el bloqueo del hilo principal.
- Implementación de validaciones estrictas de integridad del proyecto (asegurando que los medios existen y que hay clips en la línea de tiempo) antes de ejecutar tareas intensivas de renderizado.
- Dependencia principal de delegación en **FFmpeg**, usando comandos eficientes desde Rust en lugar de bindings poco mantenidos de bibliotecas de terceros.

### Formatos Soportados (Salida)
- Resolución de Pantalla: `9:16` y `16:9`
- Video: `MP4`, `MKV`, `MOV`
- Solo Audio: `MP3`, `WAV`, `AAC`, `FLAC`, `OGG`

### Formatos Soportados (Entrada)
- Video: `MP4`, `MKV`, `MOV`, `AVI`, `WEBM`
- Audio: `MP3`, `WAV`, `AAC`, `FLAC`, `OGG`, `M4A`
- Imágenes (opcional): `PNG`, `JPG`

### Capacidades del Módulo de Audio
- **Gestión de Tracks**
    - Crear track de audio con nombre, volumen base y estado muted.
    - Eliminar y renombrar tracks.
    - Ajustar volumen global del track.
    - Mutear / desmutear track.
    - Reordenar tracks (cambia prioridad en el mix).

- **Gestión de Clips (dentro de un track)**
    - Agregar clip desde archivo fuente con posición definida en la línea de tiempo (timeline_start).
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

### modulo imagen

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

- **Posicionamiento y transformación**

- Posición en el frame (x, y) — coordenadas absolutas o relativas (top-left, center, etc)
- Escala (ancho x alto, o porcentaje del frame)
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

- **Transformación**

- Escala (porcentaje o resolución exacta)
- Posición en el frame (x, y)
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