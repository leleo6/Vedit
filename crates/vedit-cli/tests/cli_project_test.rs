use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_cli_project_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let output_dir = temp_dir.path();
    let proj_path = output_dir.join("test_proj.vedit");

    // 1. Create project
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("project")
        .arg("new")
        .arg("test_proj")
        .arg("--output")
        .arg(output_dir)
        .arg("--fps")
        .arg("60.0")
        .assert()
        .success()
        .stdout(predicate::str::contains("Nuevo proyecto creado"))
        .stdout(predicate::str::contains("60"));

    // Ensure file exists
    assert!(proj_path.exists(), "El archivo de proyecto no fue creado");

    // 2. Info project
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("project")
        .arg("info")
        .arg(&proj_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Información del proyecto"))
        .stdout(predicate::str::contains("test_proj"))
        .stdout(predicate::str::contains("60 fps"));

    // 3. Open project
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("project")
        .arg("open")
        .arg(&proj_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("cargado"));

    Ok(())
}
