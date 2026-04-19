#!/bin/bash
set -e

echo "Building vedit..."
cargo build

mkdir -p tmp_test
cd tmp_test

echo "Generating test assets..."
ffmpeg -y -f lavfi -i color=c=blue:s=1920x1080:d=1 -frames:v 1 dummy.jpg -hide_banner -loglevel error
ffmpeg -y -f lavfi -i sine=frequency=440:duration=5 -c:a aac dummy.aac -hide_banner -loglevel error

echo "Creating project..."
../target/debug/vedit project new "Test Project"

echo "Adding audio track and clip..."
../target/debug/vedit track add -p test_project.vedit audio-track --kind audio
../target/debug/vedit clip add -p test_project.vedit audio-track dummy.aac

echo "Adding image track and clip..."
../target/debug/vedit track add -p test_project.vedit img-track --kind image
../target/debug/vedit image add -p test_project.vedit img-track dummy.jpg --at 0 --duration 5

echo "Rendering..."
../target/debug/vedit render full -p test_project.vedit -o final.mp4

echo "Checking output..."
ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 final.mp4
echo "Test passed!"
