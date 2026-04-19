# Vedit — Video Editor CLI

💎 **vedit** es un editor de video y audio basado estrictamente en comandos. Está diseñado de forma modular utilizando Rust, Toko y FFmpeg.

## 📦 Arquitectura

El proyecto está diseñado como un *cargo workspace* compuesto por las siguientes crates:

- **`vedit-core`**: Contiene toda la lógica agnóstica de interfaces:
  - `project`: Define las estructuras de datos (Project, Track, clips) serializados en JSON (`io.rs`).
  - `tools`: Implementaciones funcionales usando un Trait `Tool` (ej: Añadir Track, Mutear, Normalizar, etc).
  - `render`: El pipeline con soporte para FFmpeg usando el patrón Builder para generar la mezcla final o hacer un bounce de audio.
  - `history` & `context`: Manejan el historial para Undo/Redo y la sesión (estado) en memoria.
- **`vedit-cli`**: App de línea de comandos que parsea todos los sub-comandos usando `clap` de forma interactiva (con barras de progreso `indicatif` y bonitos logs con `tracing`).
- **`vedit-gui`**: *(Placeholder)* para una futura implementación usando Tauri, Dioxus o Egui.

## 🚀 Instalación y compilación

Asegúrate de tener instalados **Rust** y **FFmpeg**.

```bash
cargo build --release
```

## 🛠️ Comandos Principales

### Proyectos
```bash
vedit project new mi_proyecto --fps 60.0
vedit project info --path mi_proyecto.vedit
```

### Tracks
```bash
vedit track add -p ./mi_proyecto.vedit "Voz Off" --kind audio
vedit track volume -p ./mi_proyecto.vedit "Voz Off" 1.5
vedit track mute -p ./mi_proyecto.vedit "Voz Off"
vedit track list -p ./mi_proyecto.vedit
```

### Clips
```bash
vedit clip add -p ./mi_proyecto.vedit "Voz Off" ./mi_audio.mp3 --at 5.0
vedit clip trim -p ./mi_proyecto.vedit "Voz Off" <CLIP_ID> --start 5.0 --end 10.0
vedit clip loop -p ./mi_proyecto.vedit "Voz Off" <CLIP_ID> 4
```

### Audio (Efectos)
```bash
vedit audio normalize -p ./mi_proyecto.vedit "Voz Off" --lufs -14
vedit audio fade-in -p ./mi_proyecto.vedit "Voz Off" -d 2.0
vedit audio extract-audio -p ./mi_proyecto.vedit <VIDEO_TRACK> <CLIP_ID>
```

### Renderización 🎬
```bash
vedit render full -p ./mi_proyecto.vedit -o output.mp4 --format mp4
vedit render audio -p ./mi_proyecto.vedit -o mixdown.wav --format wav
```

## 🎬 Requisitos

* `ffmpeg` debe estar disponible en tu `$PATH` para que el módulo de renderizado y el wrapper funcionen correctamente.
