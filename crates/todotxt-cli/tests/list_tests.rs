mod helpers;

use helpers::TestFixture;
use predicates::prelude::*;

#[test]
fn list_shows_tasks() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Buy milk"));
}

#[test]
fn list_empty_filter_exits_zero() {
    let fx = TestFixture::new();
    // Filter with no matching tasks must still exit 0 (P-10)
    fx.cmd()
        .arg("list")
        .arg("xyznomatch_0000")
        .assert()
        .success();
}

#[test]
fn list_positional_filter_narrows_results() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("list")
        .arg("report")
        .assert()
        .success()
        .stdout(predicate::str::contains("report"))
        .stdout(predicate::str::contains("Buy milk").not());
}

#[test]
fn list_json_envelope() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("--json")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\":1"))
        .stdout(predicate::str::contains("\"data\""));
}

#[test]
fn list_no_color_no_ansi() {
    let fx = TestFixture::new();
    let output = fx
        .cmd()
        .arg("--no-color")
        .arg("list")
        .output()
        .expect("todotxt list ran");
    assert!(
        !output.stdout.contains(&0x1b_u8),
        "stdout contained ESC (ANSI) codes with --no-color"
    );
}
