use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;

pub struct TvaCmd {
    cmd: Command,
    stdin: Option<String>,
}

impl TvaCmd {
    pub fn new() -> Self {
        let mut cmd = cargo_bin_cmd!("tva");
        cmd.env("RUST_BACKTRACE", "1");
        Self { cmd, stdin: None }
    }

    pub fn stdin<S: Into<String>>(mut self, input: S) -> Self {
        self.stdin = Some(input.into());
        self
    }

    pub fn args(mut self, args: &[&str]) -> Self {
        self.cmd.args(args);
        self
    }

    pub fn run(mut self) -> (String, String) {
        if let Some(input) = self.stdin {
            self.cmd.write_stdin(input);
        }

        let output = self.cmd.output().expect("Failed to execute command");

        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 in stdout");
        let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8 in stderr");

        (stdout, stderr)
    }
}
