#!/bin/bash
set -e

# Configuración
VEDIT="./target/debug/vedit"
ASSETS="/home/leo/Downloads/meme"
PROJ="sequence_test"
OUTPUT="sequence_test.mp4"

echo "=== INICIANDO TEST 3 (Secuencia de videos) ==="

# 1. Crear proyecto
$VEDIT project new $PROJ --fps 30

# 2. Agregar track
$VEDIT track add -p $PROJ "Main" -k video

# 3. Agregar Video 1 (10s)
$VEDIT video add -p $PROJ "Main" "$ASSETS/pitadora-dando-vueltas-con-musicaclasica.mp4" --at 0 --duration 10 --name "Part1" > /dev/null

# 4. Agregar Video 2 (10s) empezando en 10s
$VEDIT video add -p $PROJ "Main" "$ASSETS/alce-ulle-oso-graba-desdecarro.mp4" --at 10 --duration 10 --name "Part2" > /dev/null

# 5. Renderizar
echo "Renderizando secuencia..."
$VEDIT render full -p $PROJ -o $OUTPUT --force

echo "=== TEST 3 COMPLETADO: $OUTPUT ==="
