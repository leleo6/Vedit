### Arquitectura Basica
vedit/
├── Cargo.toml
│
└── crates/
    │
    ├── vedit-core/
    │   └── src/
    │       ├── lib.rs
    │       │
    │       ├── project/               ← NUEVO, el corazón
    │       │   ├── mod.rs             # Project, ProjectMetadata
    │       │   ├── track.rs           # Track, TrackKind
    │       │   ├── clip.rs            # AudioClip, VideoClip, ImageClip
    │       │   └── io.rs              # cargar/guardar JSON en disco
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
    │       ├── render/                ← NUEVO
    │       │   ├── mod.rs             # RenderJob, RenderOutput
    │       │   ├── audio.rs           # renderiza solo audio
    │       │   ├── video.rs           # renderiza solo video
    │       │   └── compositor.rs      # mezcla todo, output final
    │       │
    │       ├── ffmpeg/
    │       │   ├── command.rs
    │       │   └── probe.rs
    │       │
    │       └── context/
    │           └── mod.rs
            └── history/           # Lógica de Undo/Redo
            ├── cache/             # Gestión de archivos temporales y proxies

    │
    ├── vedit-cli/
    │   └── src/
    │       ├── main.rs
    │       └── commands/
    │           ├── project.rs         # new, open, info
    │           ├── track.rs           # add, remove, list
    │           ├── clip.rs            # add, remove, move
    │           ├── audio.rs           # mix, mute, normalize, fade
    │           └── render.rs          # render con opciones
    │
    └── vedit-gui/
        └── src/
            └── main.rs               # placeholder

## Frameworks basicos
- clap (v4): Para el parsing del CLI con el feature de derive (muy limpio).
- indicatif: Es la librería estándar de facto para barras de progreso en Rust. Indispensable para que el usuario no piense que el CLI se congeló.
- tokio: Si planeas manejar procesos de forma asíncrona (por ejemplo, renderizar varios clips pequeños en paralelo).
- anyhow / thiserror: Para un manejo de errores robusto. En edición de video, los errores de codecs son comunes y necesitas reportarlos bien.
- tracing: Para logs. Es mucho mejor que println! porque te permite filtrar por niveles de importancia (debug, info, error).

#### Formatos de video basicos(salida)
- Pantalla |9:16 y 16:9|
- video |mp4, MKV y MOV|
- Solo audio |mp3,wav AAC FLAC y OGG|

#### Formatos de Entrada
- Video: mp4, mkv, mov, avi, webm
- Audio: mp3, wav, aac, flac, ogg, m4a

### modulo audio 
- Gestión de tracks
    - Crear track de audio con nombre, volumen base y estado muted
    - Eliminar track
    - Renombrar track
    - Ajustar volumen global del track
    - Mutear / desmutear track
    - Reordenar tracks (cambia prioridad en el mix)

- Gestión de clips dentro de un track
    - Agregar clip desde archivo fuente con posición en el timeline
    - Eliminar clip
    - Mover clip en el timeline (cambiar timeline_start)
    - Recortar clip (ajustar source_start / source_end)
    - Ajustar volumen individual del clip (independiente del track)
    - Dividir clip en un punto del timeline
    - loop — repetir un clip de audio N veces o hasta llenar una duración

- Efectos por clip
    - Fade in (duración en segundos)
    - Fade out (duración en segundos)
    - Silenciar rango de tiempo dentro del clip
    - speed/pitch

- Procesamiento por track
    - Normalizar volumen del track (nivelar loudness, estándar -23 LUFS)
    - Aplicar fade in / out al track completo

- Mezcla
    - Mezclar múltiples tracks de audio en uno solo (bounce)
    - Extraer audio de un clip de video y convertirlo en clip de audio
    - Reemplazar audio de un clip de video por otro archivo

- Render
    - Renderizar solo el audio del proyecto (only audio)
    - Renderizar audio como parte del compositor final (video + audio)


