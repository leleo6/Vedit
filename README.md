# Vedit — Video Editor CLI 🎬
 
💎 **vedit** es un Editor de Video No Lineal (NLE) completo que corre 100% en tu terminal. Diseñado de forma modular utilizando Rust, Tokio y FFmpeg, te permite editar videos con múltiples capas de audio, video, imagen y texto mediante comandos estilo Git.

A diferencia de los scripts declarativos o el uso puro de FFmpeg, **Vedit maneja estado (Stateful)**. Puedes abrir un proyecto, hacer `undo` (deshacer), `redo` (rehacer), alterar tracks de forma iterativa y renderizar el resultado final, sin jamás tener que lidiar con la arcana sintaxis de grafos de FFmpeg.

## 📦 Arquitectura

El proyecto es un *cargo workspace* compuesto por:

- **`vedit-core`**: Toda la lógica de negocio, incluyendo:
  - Manejo de `Project`, `Track` (Video, Audio, Image, Text) y `Clips`.
  - Sistema de **Undo/Redo** limitando snapshots para seguridad de memoria.
  - Generación dinámica de filtros complejos para el pipeline asíncrono sobre `tokio::process`.
- **`vedit-cli`**: Interfaz de línea de comandos interactiva construida con `clap`, trazabilidad con `tracing`, y barras de progreso con `indicatif`.

## 🚀 Instalación y compilación

Requisitos:
- **Rust** (edición 2021)
- **FFmpeg** y **ffprobe** (disponibles en tu `$PATH`)

```bash
cargo build --release
```
El ejecutable resultante estará en `target/release/vedit` y pesa apenas unos megabytes.

## 🛠️ Flujo de Trabajo y Comandos

### Proyectos e Historial
Mantén un registro de todo tu progreso y revierte errores fácilmente.
```bash
vedit project new mi_proyecto --fps 60.0
vedit project info --path mi_proyecto.vedit
vedit project undo --path mi_proyecto.vedit
vedit project redo --path mi_proyecto.vedit
```

### Gestión de Tracks
Los tracks agrupan clips. Soportamos `audio`, `video`, `image`, y `text`.
```bash
vedit track add -p mi_proyecto.vedit "Voz Off" --kind audio
vedit track add -p mi_proyecto.vedit "Subtítulos" --kind text
vedit track volume -p mi_proyecto.vedit "Voz Off" 1.5
vedit track list -p mi_proyecto.vedit
```

### Manipulación de Clips Multimedia
Posiciona elementos en la línea de tiempo.
```bash
# Añadir audio
vedit clip add -p mi_proyecto.vedit "Voz Off" ./mi_audio.mp3 --at 5.0
vedit clip trim -p mi_proyecto.vedit "Voz Off" <CLIP_ID> --start 5.0 --end 10.0

# Añadir imágenes (Ej: overlay de un logo de 5 segundos)
vedit image add -p mi_proyecto.vedit "CapaLogo" ./logo.png --at 0.0 --duration 5.0

# Añadir texto estático
vedit text add -p mi_proyecto.vedit "Subtítulos" "Clip1" "Hola Mundo desde Vedit!" --at 2.0 --duration 4.0
```

### Efectos y Procesamiento de Audio
```bash
vedit audio normalize -p mi_proyecto.vedit "Voz Off" --lufs -14
vedit audio fade-in -p mi_proyecto.vedit "Voz Off" -d 2.0
vedit audio extract-audio -p mi_proyecto.vedit <VIDEO_TRACK> <CLIP_ID>
```

### Renderización Final 🍿
Vedit compila automáticamente el grafo de FFmpeg (manejo de escalas, posiciones, fundidos, mezclas multicanal) y te entrega el resultado:
```bash
# Render de Audio+Video Final
vedit render full -p mi_proyecto.vedit -o output.mp4 --format mp4

# Previsualizar solo el texto (rápido, sin procesar videos pesados)
vedit render text-preview -p mi_proyecto.vedit -o preview.mp4

# Renderizar solo el Mixdown de audio
vedit render audio -p mi_proyecto.vedit -o mixdown.wav --format wav
```

## 🛡️ Estabilidad y Tests
Vedit posee una arquitectura altamente defensiva y validada:
- No falla ante rutas de archivo faltantes o no-UTF8 (se rechaza en validación).
- Límite de fuga de memoria en la pila del Undo/Redo.
- Operaciones asíncronas, evitando que la interfaz se congele.
- Conjunto de test exhaustivos integrados (`cargo test --workspace`).
