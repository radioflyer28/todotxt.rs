use assert_cmd::Command;
use assert_fs::{fixture::ChildPath, prelude::*, TempDir};

pub const SAMPLE_TODO: &str = "(A) Buy milk +groceries @home\n\
(B) Send report +work @office\n\
x 2024-01-01 Done task +work\n\
Call dentist @personal\n";

/// A temporary filesystem fixture with a pre-populated todo.txt and config.toml.
///
/// `dir` and `todo` are kept as fields to ensure `TempDir` stays alive for the
/// duration of the test and to allow tests to inspect the fixture files directly.
#[allow(dead_code)]
pub struct TestFixture {
    pub dir: TempDir,
    pub todo: ChildPath,
    pub config: ChildPath,
}

impl TestFixture {
    pub fn new() -> Self {
        Self::with_content(SAMPLE_TODO)
    }

    pub fn with_content(content: &str) -> Self {
        let dir = TempDir::new().expect("create TempDir");
        let todo = dir.child("todo.txt");
        todo.write_str(content).expect("write todo.txt");
        let config = dir.child("config.toml");
        // Use Debug format so Windows backslashes are escaped correctly in TOML
        config
            .write_str(&format!("todo_file = {:?}\n", todo.path()))
            .expect("write config.toml");
        TestFixture { dir, todo, config }
    }

    /// Build a `Command` for the `todotxt` binary, pre-configured with `--config`.
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("todotxt").expect("todotxt binary found");
        cmd.arg("--config").arg(self.config.path());
        cmd
    }
}
