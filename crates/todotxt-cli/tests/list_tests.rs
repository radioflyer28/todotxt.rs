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

// ── Gap-closure regression tests (03-05) ─────────────────────────────────────

/// Default list must exclude completed tasks (UAT tests 2, 10, 12).
#[test]
fn list_default_excludes_completed_tasks() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Buy milk"))
        .stdout(predicate::str::contains("Done task").not());
}

/// Explicit DONE token shows only completed tasks.
#[test]
fn list_done_token_shows_completed_tasks() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("list")
        .arg("DONE")
        .assert()
        .success()
        .stdout(predicate::str::contains("Done task"))
        .stdout(predicate::str::contains("Buy milk").not());
}

/// Unknown preset emits warning to stderr, exits 0, and still applies default -DONE filter.
#[test]
fn list_unknown_preset_warns_on_stderr_exits_zero() {
    let fx = TestFixture::new();
    let output = fx
        .cmd()
        .arg("list")
        .arg(":nonexistent")
        .output()
        .expect("todotxt list ran");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "exit code must be 0");
    assert!(
        stderr.contains("warning: unknown preset ':nonexistent'"),
        "warning not found in stderr: {stderr}"
    );
    // Default -DONE semantics must still apply even after unknown preset.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Done task"),
        "completed task must be excluded from default list; stdout: {stdout}"
    );
}

/// JSON output must not contain carriage-return characters in any field (UAT test 9).
#[test]
fn list_json_no_cr_in_output() {
    // Use CRLF content to exercise the CR-normalization path.
    let fx = TestFixture::with_content(
        "(A) Buy milk +groceries @home\r\nx 2024-01-01 Done task +work\r\n",
    );
    let output = fx
        .cmd()
        .arg("--json")
        .arg("list")
        .arg("DONE") // request completed task so we can inspect its raw field
        .output()
        .expect("todotxt --json list ran");
    assert!(output.status.success());
    assert!(
        !output.stdout.contains(&b'\r'),
        "JSON stdout must not contain bare carriage-return bytes"
    );
    // serde_json escapes \r as \\r; neither form should appear.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("\\r"),
        "JSON stdout must not contain \\r escape sequences; got: {stdout}"
    );
}

/// --no-color output must be free of ANSI escapes and carriage-return artifacts (UAT test 10).
#[test]
fn list_no_color_no_cr_artifacts() {
    // Use CRLF content to exercise the CR-normalization path.
    let fx =
        TestFixture::with_content("(A) Buy milk +groceries @home\r\nCall dentist @personal\r\n");
    let output = fx
        .cmd()
        .arg("--no-color")
        .arg("list")
        .output()
        .expect("todotxt --no-color list ran");
    assert!(output.status.success());
    assert!(
        !output.stdout.contains(&0x1b_u8),
        "stdout must not contain ANSI escape codes with --no-color"
    );
    assert!(
        !output.stdout.contains(&b'\r'),
        "stdout must not contain carriage-return characters"
    );
}

#[test]
fn list_positional_or_context_filter_matches_either_context() {
    let fx = TestFixture::with_content(
        "(A) Alpha task @work\n(B) Beta task @home\n(C) Gamma task @gym\n",
    );
    let output = fx
        .cmd()
        .arg("list")
        .arg("@work|@home")
        .output()
        .expect("todotxt list positional OR ran");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Alpha task"),
        "work task must be included: {stdout}"
    );
    assert!(
        stdout.contains("Beta task"),
        "home task must be included: {stdout}"
    );
    assert!(
        !stdout.contains("Gamma task"),
        "gym task must be excluded: {stdout}"
    );
}

#[test]
fn list_filter_flag_or_priority_matches_either_priority() {
    let fx = TestFixture::with_content(
        "(A) Alpha task @work\n(B) Beta task @home\n(C) Gamma task @gym\n",
    );
    let output = fx
        .cmd()
        .arg("list")
        .arg("--filter")
        .arg("(A)|(B)")
        .output()
        .expect("todotxt list --filter OR ran");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Alpha task"),
        "A task must be included: {stdout}"
    );
    assert!(
        stdout.contains("Beta task"),
        "B task must be included: {stdout}"
    );
    assert!(
        !stdout.contains("Gamma task"),
        "C task must be excluded: {stdout}"
    );
}

#[test]
fn list_or_and_combined_requires_both_terms() {
    let fx = TestFixture::with_content(
        "(A) Alpha task @work\n(B) Beta task @work\n(B) Beta home task @home\n(C) Gamma task @work\n",
    );
    let output = fx
        .cmd()
        .arg("list")
        .arg("(A)|(B)")
        .arg("@work")
        .output()
        .expect("todotxt list OR+AND ran");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Alpha task"),
        "A @work task must be included: {stdout}"
    );
    assert!(
        stdout.contains("Beta task @work"),
        "B @work task must be included: {stdout}"
    );
    assert!(
        !stdout.contains("Beta home task"),
        "B @home task must be excluded: {stdout}"
    );
    assert!(
        !stdout.contains("Gamma task"),
        "C @work task must be excluded: {stdout}"
    );
}

#[test]
fn list_or_empty_branch_is_tolerated() {
    let fx = TestFixture::with_content("(A) Alpha task @work\n(B) Beta task @home\n");
    let output = fx
        .cmd()
        .arg("list")
        .arg("@work|")
        .output()
        .expect("todotxt list empty OR branch ran");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Alpha task"),
        "work task must be included: {stdout}"
    );
    assert!(
        !stdout.contains("Beta task"),
        "home task must be excluded: {stdout}"
    );
}
