#!/bin/bash
set -e

# Configuración
VEDIT="./target/debug/vedit"
ASSETS="/home/leo/Downloads/meme"
INPUT_V="portrait_test.mp4"
PROJ_W="widescreen_test"
OUTPUT_W="widescreen_test.mp4"

echo "=== INICIANDO TEST 2 (16:9, 5m) ==="

# 1. Crear proyecto
$VEDIT project new $PROJ_W --fps 30

# 2. Agregar track para las columnas
$VEDIT track add -p $PROJ_W "Columns" -k video

# 3. Agregar el video anterior (Columna Izquierda - Normal)
for i in {0..3}; do
    AT=$((i * 30))
    $VEDIT video add -p $PROJ_W "Columns" "$INPUT_V" --at $AT --name "Left_$i" > /dev/null
    # Obtener el ID del clip recién agregado (buscando por nombre)
    CLIP_ID=$($VEDIT video list -p $PROJ_W "Columns" | grep "Left_$i" -A 1 | grep -oP 'id: \K[0-9a-f-]+')
    $VEDIT video transform -p $PROJ_W "Columns" $CLIP_ID --scale-w 0.4 --scale-h 0.8 --x 0.05 --y 0.1
done

# 4. Agregar el video anterior (Columna Derecha - Inversa)
for i in {0..3}; do
    AT=$((i * 30))
    $VEDIT video add -p $PROJ_W "Columns" "$INPUT_V" --at $AT --name "Right_$i" > /dev/null
    CLIP_ID=$($VEDIT video list -p $PROJ_W "Columns" | grep "Right_$i" -A 1 | grep -oP 'id: \K[0-9a-f-]+')
    $VEDIT video transform -p $PROJ_W "Columns" $CLIP_ID --scale-w 0.4 --scale-h 0.8 --x 0.55 --y 0.1
    $VEDIT video speed -p $PROJ_W "Columns" $CLIP_ID --reverse
done

# 5. Agregar un fondo y otros materiales para llegar a los 2 minutos (120s)
$VEDIT track add -p $PROJ_W "Background" -k video
$VEDIT video add -p $PROJ_W "Background" "$ASSETS/alce-ulle-oso-graba-desdecarro.mp4" --duration 120 --at 0 > /dev/null

# 6. Agregar Audio con Fade
$VEDIT track add -p $PROJ_W "Music" -k audio
AUDIO_ID=$($VEDIT clip add -p $PROJ_W "Music" "$ASSETS/Skillet - Falling Inside The Black [HQ] - Christianmuzikz (128k).mp3" --at 0 | grep -oP 'id: \K[0-9a-f-]+')
$VEDIT clip trim -p $PROJ_W --start 0 --end 120 "Music" $AUDIO_ID
$VEDIT audio fade-in -p $PROJ_W "Music" --clip $AUDIO_ID --duration 5

# 7. Texto informativo
$VEDIT text add-track -p $PROJ_W "UI"
$VEDIT text add -p $PROJ_W --at 0 --duration 120 "UI" "info" "Test 16:9 - Columnas Duales + Audio"

# 7. Renderizar (Usamos una resolución menor para que no tarde una eternidad en el test si es posible, 
# pero el comando full usará la del proyecto)
echo "Renderizando video 16:9 (5 minutos)..."
# Nota: Esto puede tardar. Para el test rápido, podrías limitar el tiempo, 
# pero el usuario pidió 5 min.
$VEDIT render full -p $PROJ_W -o $OUTPUT_W --force

echo "=== TEST 2 COMPLETADO: $OUTPUT_W ==="
