use std::{
    io,
    process::{Command, ExitCode, Output, Stdio, Termination},
};

fn main() -> io::Result<OutputExt> {
    #[cfg(unix)]
    {
        const SCRIPT: &str = r#"
            current=$(git branch --show-current) &&

            default=$(git remote show origin | awk '/HEAD branch/ {print $NF}') &&
            
            git checkout "${default}" &&
            git pull --ff-only origin || git pull --ff-only upstream &&
            git branch -D "${current}" &&
            git push origin --delete "${current}"
        "#;

        Command::new("sh")
            .args(["-c", SCRIPT])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map(OutputExt)
    }

    #[cfg(windows)]
    {
        const SCRIPT: &str = r#"
            $current = (git branch --show-current)
            if (-not $current) { exit $LASTEXITCODE }

            $default = (git remote show origin | Select-String 'HEAD branch') -replace '.*: '
            if (-not $default) { exit $LASTEXITCODE }

            function Run($cmd) {
                & $cmd
                if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            }
            Run { git checkout $default }
            Run {
                git pull --ff-only origin
                if ($LASTEXITCODE -ne 0) {
                    git pull --ff-only upstream
                }
            }
            Run { git branch -D $current }
            Run { git push origin --delete $current }
        "#;

        Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                SCRIPT,
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
