mod helpers;

use helpers::TestFixture;
use predicates::prelude::*;

#[test]
fn stats_shows_counts() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total:"))
        .stdout(predicate::str::contains("Complete:"))
        .stdout(predicate::str::contains("Incomplete:"));
}

#[test]
fn stats_json_has_total_key() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("--json")
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema_version\":1"))
        .stdout(predicate::str::contains("\"total\""));
}
