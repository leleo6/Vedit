use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_cli_track_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let proj_path = temp_dir.path().join("track_proj.vedit");

    // 1. Create project
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("project")
        .arg("new")
        .arg("track_proj")
        .arg("--output")
        .arg(temp_dir.path())
        .assert()
        .success();

    // 2. Add tracks
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("add")
        .arg("--project")
        .arg(&proj_path)
        .arg("--kind")
        .arg("video")
        .arg("Main Video")
        .assert()
        .success()
        .stdout(predicate::str::contains("agregado"));

    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("add")
        .arg("--project")
        .arg(&proj_path)
        .arg("--kind")
        .arg("audio")
        .arg("Background Music")
        .assert()
        .success();

    // 3. List tracks
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("list")
        .arg("--project")
        .arg(&proj_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Main Video"))
        .stdout(predicate::str::contains("Background Music"));

    // 4. Rename track
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("rename")
        .arg("--project")
        .arg(&proj_path)
        .arg("Main Video")
        .arg("A-Roll")
        .assert()
        .success()
        .stdout(predicate::str::contains("renombrado"));

    // Verify rename
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("list")
        .arg("--project")
        .arg(&proj_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("A-Roll"))
        .stdout(predicate::str::contains("Main Video").not());

    // 5. Volume and Mute
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("volume")
        .arg("--project")
        .arg(&proj_path)
        .arg("A-Roll")
        .arg("0.5")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("mute")
        .arg("--project")
        .arg(&proj_path)
        .arg("Background Music")
        .assert()
        .success();

    // 6. Remove track
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("remove")
        .arg("--project")
        .arg(&proj_path)
        .arg("Background Music")
        .assert()
        .success()
        .stdout(predicate::str::contains("eliminado"));

    // List again to confirm removal
    let mut cmd = Command::cargo_bin("vedit")?;
    cmd.arg("track")
        .arg("list")
        .arg("--project")
        .arg(&proj_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Background Music").not());

    Ok(())
}
