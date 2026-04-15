mod helpers;

use helpers::{TestFixture, SAMPLE_TODO};
use predicates::prelude::*;

#[test]
fn show_first_task_prints_raw_line() {
    let fx = TestFixture::new();
    let first_line = SAMPLE_TODO.lines().next().expect("SAMPLE_TODO has lines");
    fx.cmd()
        .arg("show")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains(first_line));
}

#[test]
fn show_nonexistent_id_exits_one() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("show")
        .arg("9999")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn show_zero_id_exits_one() {
    let fx = TestFixture::new();
    // ID 0 is invalid (IDs are 1-based); show.rs returns CliError::NotFound for id==0
    fx.cmd()
        .arg("show")
        .arg("0")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn show_json_not_found_has_error_key() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("--json")
        .arg("show")
        .arg("9999")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("\"schema_version\":1"))
        .stdout(predicate::str::contains("\"error\""));
}
