#!/bin/bash
set -e

# Configuración
VEDIT="./target/debug/vedit"
ASSETS="/home/leo/Downloads/meme"
PROJ_P="portrait_test"
OUTPUT_P="portrait_test.mp4"

echo "=== INICIANDO TEST 1 (9:16, 30s) ==="

# 1. Crear proyecto
$VEDIT project new $PROJ_P --fps 30

# 2. Agregar track de video y clip principal
$VEDIT track add -p $PROJ_P "Main Video" -k video
CLIP_ID=$($VEDIT video add -p $PROJ_P "Main Video" "$ASSETS/pitadora-dando-vueltas-con-musicaclasica.mp4" --duration 30 | grep -oP 'id: \K[0-9a-f-]+')

# 3. Aplicar filtros de video
$VEDIT video effects -p $PROJ_P "Main Video" $CLIP_ID --blur 5 --vignette 0.5
$VEDIT video color -p $PROJ_P "Main Video" $CLIP_ID --brightness 0.1 --saturation 1.5

# 4. Agregar Stickers (Imagen) con movimiento (Ken Burns)
$VEDIT track add -p $PROJ_P "Sticker" -k image
STICKER_ID=$($VEDIT image add -p $PROJ_P "Sticker" "$ASSETS/gatomeawin-levantandoceja.png" --at 5 --duration 10 | grep -oP 'id: \K[0-9a-f-]+')
# $VEDIT image ken-burns -p $PROJ_P "Sticker" $STICKER_ID

# 5. Agregar Audio con filtro (Fade)
$VEDIT track add -p $PROJ_P "Music" -k audio
AUDIO_ID=$($VEDIT clip add -p $PROJ_P "Music" "$ASSETS/Skillet - Falling Inside The Black [HQ] - Christianmuzikz (128k).mp3" --at 0 | grep -oP 'id: \K[0-9a-f-]+')
$VEDIT clip trim -p $PROJ_P --start 0 --end 30 "Music" $AUDIO_ID
$VEDIT audio fade-in -p $PROJ_P "Music" --clip $AUDIO_ID --duration 3

# 6. Agregar Textos informativos de los efectos
$VEDIT text add-track -p $PROJ_P "Info"
$VEDIT text add -p $PROJ_P --at 0 --duration 5 "Info" "txt1" "TEST VEDIT 9:16 - Blur + Vignette"
$VEDIT text add -p $PROJ_P --at 5 --duration 10 "Info" "txt2" "Efecto Ken Burns en Sticker"
$VEDIT text add -p $PROJ_P --at 15 --duration 5 "Info" "txt3" "Saturation 1.5 + Brightness 0.1"
$VEDIT text add -p $PROJ_P --at 25 --duration 5 "Info" "txt4" "Renderizando Final..."

# 7. Renderizar
echo "Renderizando video 9:16..."
VEDIT_LOG=debug $VEDIT render full -p $PROJ_P -o $OUTPUT_P --aspect 9:16 --force

echo "=== TEST 1 COMPLETADO: $OUTPUT_P ==="
