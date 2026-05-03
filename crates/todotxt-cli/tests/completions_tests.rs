mod helpers;

use helpers::TestFixture;
use predicates::prelude::*;

#[test]
fn completions_bash_produces_output() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh_produces_output() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("completions")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}
