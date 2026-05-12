use std::{
    env,
    io::{self, Error, ErrorKind},
    process::{Command, ExitCode, Output, Stdio, Termination},
};

#[allow(clippy::too_many_lines)]
fn main() -> io::Result<OutputExt> {
    let mut args = env::args().skip(1);
    let branch = match args.next() {
        Some(flag) if flag == "-b" => {
            let branch = args.next().ok_or_else(|| {
                Error::new(ErrorKind::InvalidInput, "missing branch name after -b")
            })?;
            if args.next().is_some() {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    "unexpected extra arguments",
                ));
            }
            Some(branch)
        }
        Some(_) => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "unexpected argument, expected -b",
            ));
        }
        None => None,
    };
    let branch = branch.as_deref().unwrap_or_default();

    #[cfg(unix)]
    {
        const SCRIPT: &str = r#"
            current="${GIT_SAO_BRANCH}" &&
            if [ "${current}" ]; then
                git checkout "${current}"
            fi &&

            current=$(git branch --show-current) &&

            default=$(git remote show origin | awk '/HEAD branch/ {print $NF}') &&

            git checkout "${default}" &&

            remote=$(git config branch."${default}".remote) &&
            if [ "${remote}" = "origin" ]; then
                git pull --ff-only origin
            else
                git pull --ff-only upstream
            fi &&

            git branch -D "${current}" &&
            if ! git push origin --delete "${current}"; then
                for remote in $(git remote); do
                    # skip remote that is origin
                    if [ "${remote}" = "origin" ]; then
                        continue
                    fi
                    if git ls-remote --heads "${remote}" "${current}"; then
                        git push "${remote}" --delete "${current}" && break
                    fi
                done
            fi
        "#;

        Command::new("sh")
            .args(["-c", SCRIPT])
            .env("GIT_SAO_BRANCH", branch)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .map(OutputExt)
    }

    #[cfg(windows)]
    {
        const SCRIPT: &str = r#"
            current = $env:GIT_SAO_BRANCH
            if ($current) {
                git checkout $current
            }

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
                $remote = (git config branch."$default".remote)
                if ($remote -eq "origin") {
                    git pull --ff-only origin
                } else {
                    git pull --ff-only upstream
                }
            }

            Run { git branch -D $current }

            try {
                Run { git push origin --delete $current }
            } catch {
                foreach ($remote in (git remote)) {
                    # skip remote that is origin
                    if ($remote -eq "origin") {
                        continue
                    }
                    if (git ls-remote --heads $remote $current) {
                        Run { git push $remote --delete $current }
                        if ($LASTEXITCODE -eq 0) { break }
                    }
                }
            }
        "#;

        Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                SCRIPT,
            ])
            .env("GIT_SAO_BRANCH", branch)
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
