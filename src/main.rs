use std::{
    io,
    process::{Command, ExitCode, Output, Stdio, Termination},
};

fn main() -> io::Result<OutputExt> {
    #[cfg(unix)]
    {
        let script = r#"
        current=$(git branch --show-current) &&
        default=$(git symbolic-ref --short refs/remotes/origin/HEAD) &&
        default=${default#origin/} &&
        git checkout "$default" &&
        git pull --ff-only origin &&
        git branch -D "$current" &&
        git push origin --delete "$current"
    "#;

        Command::new("sh")
            .args(["-c", script])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map(OutputExt)
    }

    #[cfg(windows)]
    {
        let script = r#"
            $current = (git branch --show-current).Trim()
            if (-not $current) { exit $LASTEXITCODE }

            $default = (git symbolic-ref --short refs/remotes/origin/HEAD).Trim()
            if (-not $default) { exit $LASTEXITCODE }
            $default = $default -replace '^origin/', ''

            function Run($cmd) {
                & $cmd
                if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            }
            Run { git checkout $default }
            Run { git pull --ff-only origin }
            Run { git branch -D $current }
            Run { git push origin --delete $current }
        "#;

        Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script,
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map(OutputExt)
    }
}

// std::process::Output lacks Termination impl
struct OutputExt(Output);

impl Termination for OutputExt {
    fn report(self) -> ExitCode {
        if self.0.status.success() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        }
    }
}
