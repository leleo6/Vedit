use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_render_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let proj_dir = temp.path().join("render_test");
    fs::create_dir_all(&proj_dir)?;

    let proj_file = proj_dir.join("test_render.vedit");
    let image_file = proj_dir.join("dummy.png");
    let output_file = proj_dir.join("output.mp4");

    // 1. Crear una imagen dummy para test usando ffmpeg
    std::process::Command::new("ffmpeg")
        .args([
            "-f", "lavfi",
            "-i", "color=c=red:s=640x480",
            "-frames:v", "1",
            image_file.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to create dummy image with ffmpeg");

    // 2. Crear proyecto
    Command::cargo_bin("vedit")?
        .args(["project", "new", "Test Render", "--output", proj_dir.to_str().unwrap()])
        .assert()
        .success();

    // 3. Crear un track de imagen y agregar clip
    Command::cargo_bin("vedit")?
        .args(["track", "add", "ImageTrack", "--kind", "image", "--project", proj_file.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("vedit")?
        .args([
            "image", "add", "ImageTrack",
            image_file.to_str().unwrap(),
            "--at", "0.0", // timeline_start
            "--duration", "2.0",
            "--project", proj_file.to_str().unwrap()
        ])
        .assert()
        .success();

    // 4. Crear un track de texto y agregar clip
    Command::cargo_bin("vedit")?
        .args(["track", "add", "TextTrack", "--kind", "text", "--project", proj_file.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("vedit")?
        .args([
            "text", "add", "TextTrack",
            "Text1", // nombre del clip
            "Hola Mundo desde Vedit!",
            "--at", "0.0", // timeline start
            "--duration", "2.0", // duration
            "--project", proj_file.to_str().unwrap()
        ])
        .assert()
        .success();

    // 5. Renderizar el proyecto
    Command::cargo_bin("vedit")?
        .args([
            "render", "full",
            "--output", output_file.to_str().unwrap(),
            "--project", proj_file.to_str().unwrap()
        ])
        .assert()
        .success();

    // 6. Verificar que el output existe y no está vacío
    assert!(output_file.exists(), "El archivo de salida de renderizado no fue creado");
    let meta = fs::metadata(&output_file)?;
    assert!(meta.len() > 1000, "El archivo renderizado parece demasiado pequeño o corrupto");

    Ok(())
}
