use std::{
    io,
    process::{Command, ExitCode, Output, Stdio, Termination},
};

fn main() -> io::Result<OutputExt> {
    #[cfg(unix)]
    {
        const SCRIPT: &str = r#"
            current=$(git branch --show-current) &&

            default=$(git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null || git symbolic-ref --short refs/remotes/upstream/HEAD) &&
            case ${default} in
            origin/*) default=${default#origin/} ;;
            *) default=${default#upstream/} ;;
            esac &&
            
            git checkout "${default}" &&
            git pull --ff-only origin 2>/dev/null || git pull --ff-only upstream &&
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
            $current = (git branch --show-current).Trim()
            if (-not $current) { exit $LASTEXITCODE }

            $default = (git symbolic-ref --short refs/remotes/origin/HEAD 2>$null)
            if ($LASTEXITCODE -ne 0 -or -not $default) {
                $default = (git symbolic-ref --short refs/remotes/upstream/HEAD)
            }
            $default = $default.Trim() -replace '^origin/', ''

            function Run($cmd) {
                & $cmd
                if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
            }
            Run { git checkout $default }
            Run {
                git pull --ff-only origin 2>$null
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
