<div align="center">

```text
 ██╗   ██╗███████╗██████╗ ██╗████████╗
 ██║   ██║██╔════╝██╔══██╗██║╚══██╔══╝
 ██║   ██║█████╗  ██║  ██║██║   ██║   
 ╚██╗ ██╔╝██╔══╝  ██║  ██║██║   ██║   
  ╚████╔╝ ███████╗██████╔╝██║   ██║   
   ╚═══╝  ╚══════╝╚═════╝ ╚═╝   ╚═╝   
```
**Video Editor CLI 🎬**

</div>

---

💎 **vedit** es un Editor de Video No Lineal (NLE) completo que corre 100% en tu terminal. Diseñado de forma modular utilizando Rust, Tokio y FFmpeg, te permite editar videos con múltiples capas de audio, video, imagen y texto mediante comandos estilo Git.

A diferencia de los scripts declarativos o el uso puro de FFmpeg, **Vedit maneja estado (Stateful)**. Puedes abrir un proyecto, hacer `undo` (deshacer), `redo` (rehacer), alterar tracks de forma iterativa y renderizar el resultado final, sin jamás tener que lidiar con la arcana sintaxis de grafos de FFmpeg.

## 📦 Arquitectura

El proyecto es un *cargo workspace* compuesto por:

- **`vedit-core`**: Toda la lógica de negocio, incluyendo:
  - Manejo de `Project`, `Track` (Video, Audio, Image, Text) y `Clips`.
  - Sistema de **Undo/Redo** limitando snapshots para seguridad de memoria.
  - Generación dinámica de filtros complejos para el pipeline asíncrono sobre `tokio::process`.
- **`vedit-cli`**: Interfaz de línea de comandos interactiva construida con `clap`, trazabilidad con `tracing`, y barras de progreso con `indicatif`.

### 1. Instalar dependencias (FFmpeg)

Vedit requiere **FFmpeg** y **ffprobe** instalados en el sistema.

| Distribución | Comando de instalación |
| :--- | :--- |
| **Ubuntu / Debian** | `sudo apt update && sudo apt install ffmpeg` |
| **Fedora** | `sudo dnf install ffmpeg-free` (o `ffmpeg` desde RPMFusion) |
| **Arch Linux** | `sudo pacman -S ffmpeg` |
| **macOS (Homebrew)** | `brew install ffmpeg` |
| **Windows (Winget)** | `winget install ffmpeg` |

### 2. Compilar Vedit

Requiere **Rust** (edición 2021). Si no lo tienes, instálalo vía [rustup.rs](https://rustup.rs/).

```bash
cargo build --release
```
El ejecutable resultante estará en `target/release/vedit`. Puedes moverlo a tu `/usr/local/bin` para usarlo globalmente.

## 🛠️ Flujo de Trabajo y Comandos

### Proyectos e Historial
Vedit ahora gestiona proyectos como directorios. Al crear uno, se genera una carpeta con una base de datos oculta en `.vedit/`.
```bash
vedit project new mi_proyecto --fps 30.0
vedit project info -p mi_proyecto
vedit project timeline -p mi_proyecto
vedit project undo -p mi_proyecto
vedit project redo -p mi_proyecto
```

### Gestión de Tracks
Los tracks agrupan clips. Soportamos `audio`, `video`, `image`, y `text`.
```bash
vedit track add -p mi_proyecto "Voz Off" --kind audio
vedit track add -p mi_proyecto "Subtítulos" --kind text
vedit track volume -p mi_proyecto "Voz Off" 1.5
vedit track list -p mi_proyecto
```

### Manipulación de Clips Multimedia
Posiciona elementos en la línea de tiempo.
```bash
# Añadir audio (con duración explícita opcional para omitir ffprobe)
vedit clip add -p mi_proyecto "Voz Off" ./mi_audio.mp3 --at 5.0 --duration 10.0
vedit clip trim -p mi_proyecto "Voz Off" <CLIP_ID> --start 5.0 --end 10.0

# Añadir imágenes (Ej: overlay de un logo de 5 segundos)
vedit image add -p mi_proyecto "CapaLogo" ./logo.png --at 0.0 --duration 5.0

# Añadir texto estático
vedit text add -p mi_proyecto "Subtítulos" "Clip1" "Hola Mundo desde Vedit!" --at 2.0 --duration 4.0
```

### Efectos y Procesamiento de Audio
```bash
vedit audio normalize -p mi_proyecto "Voz Off" --lufs -14
vedit audio fade-in -p mi_proyecto "Voz Off" -d 2.0
vedit audio extract-audio -p mi_proyecto <VIDEO_TRACK> <CLIP_ID>
```

### Renderización Final 🍿
Vedit compila automáticamente el grafo de FFmpeg (manejo de escalas, posiciones, fundidos, mezclas multicanal, sincronización de FPS y PTS) y entrega el resultado:
```bash
# Render de Audio+Video Final
vedit render full -p mi_proyecto -o output.mp4 --format mp4

# Previsualizar solo el texto (rápido, sin procesar videos pesados)
vedit render text-preview -p mi_proyecto -o preview.mp4

# Renderizar solo el Mixdown de audio
vedit render audio -p mi_proyecto -o mixdown.wav --format wav
```

### Configuración Global ⚙️
Vedit permite personalizar el entorno y la aceleración por hardware a nivel global:
```bash
# Ver configuración actual
vedit config show

# Usar NVIDIA NVENC o Intel VA-API por defecto para renders más rápidos
vedit config set preferred-encoder h264_nvenc

# Cambiar FPS y resolución predeterminada para nuevos proyectos
vedit config set default-fps 60
vedit config set default-resolution 1920x1080

# Diagnóstico del entorno (verifica FFmpeg y HW accel)
vedit config check
```

## 🛡️ Estabilidad y Tests
Vedit posee una arquitectura altamente defensiva y validada:
- No falla ante rutas de archivo faltantes o no-UTF8 (se rechaza en validación).
- Límite de fuga de memoria en la pila del Undo/Redo.
- Operaciones asíncronas, evitando que la interfaz se congele.
- Conjunto de test exhaustivos integrados (`cargo test --workspace`).
