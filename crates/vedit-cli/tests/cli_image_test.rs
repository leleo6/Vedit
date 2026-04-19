use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_cli_image_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let proj_path = temp_dir.path().join("test_proj.vedit");

    // 1. Create project
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("project")
        .arg("new")
        .arg("test_proj")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // 2. Add video track
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("add")
        .arg("--project")
        .arg(&proj_path)
        .arg("--kind")
        .arg("video")
        .arg("MyVideoTrack")
        .assert()
        .success();

    // Create a dummy image file
    let img_path = temp_dir.path().join("dummy.png");
    std::fs::write(&img_path, b"fake image content")?;

    // 3. Add image clip
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("image")
        .arg("add")
        .arg("--project")
        .arg(&proj_path)
        .arg("MyVideoTrack")
        .arg(&img_path)
        .arg("--at")
        .arg("2.5")
        .arg("--duration")
        .arg("10.0")
        .arg("--name")
        .arg("my_test_image")
        .assert()
        .success()
        .stdout(predicate::str::contains("agregado"));

    // 4. List image clips
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("image")
        .arg("list")
        .arg("--project")
        .arg(&proj_path)
        .arg("MyVideoTrack")
        .assert()
        .success()
        .stdout(predicate::str::contains("my_test_image"))
        .stdout(predicate::str::contains("2.50s"));

    // 5. Transform image clip (we need the UUID, but since there's only one, let's just 
    // test the command parsing for now by checking failure without UUID or try to parse list)
    // Actually, we can test failure of transform with invalid UUID
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("image")
        .arg("transform")
        .arg("--project")
        .arg(&proj_path)
        .arg("MyVideoTrack")
        .arg("00000000-0000-0000-0000-000000000000") // invalid UUID
        .arg("--opacity")
        .arg("0.5")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no encontrado"));

    Ok(())
}
