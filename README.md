The `action-validator` is a standalone tool designed to "lint" the YAML files
used to define GitHub Actions and Workflows.  It ensures that they are well-formed,
by checking them against published JSON schemas, and it makes sure that any
globs used in `paths` / `paths-ignore` match at least one file in the repo.

The intended use case for `action-validator` is in Git pre-commit hooks and
similar situations.


# Installation

The [GitHub releases](https://github.com/mpalmer/action-validator/releases)
have some pre-built binaries -- just download and put them in your path.  If a
binary for your platform isn't available, let me know and I'll see what I can
figure out.  If you want to build locally, you'll need to install a
[Rust](https://rust-lang.org) toolchain and then run `cargo build`.


## Using asdf

If you're a proponent of the [asdf tool](https://asdf-vm.com/), then you can
use that to install and manage `action-validator`:

```shell
asdf plugin add action-validator
# or
asdf plugin add action-validator https://github.com/mpalmer/action-validator.git
```

Install/configure action-validator:

```shell
# Show all installable versions
asdf list-all action-validator

# Install specific version
asdf install action-validator latest

# Set a version globally (on your ~/.tool-versions file)
asdf global action-validator latest

# Now action-validator commands are available
action-validator --version
```


# Usage

Couldn't be simpler: just pass a file to the program:

```
action-validator .github/workflows/build.yml
```

Use `action-validator -h` to see additional options.

> ### CAUTION
>
> As the intended use-case for `action-validator` is in pre-commit hooks,
> it assumes that it is being run from the root of the repository.  Glob
> checking will explode horribly if you run it from a sub-directory of the
> repo -- or, heaven forfend, outside the repository entirely.


## Pre-commit hook example

Create an executable file in the .git/hooks directory of the target repository:
`touch .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit` and paste the following example code:

```bash
#!/bin/bash
if [ -d ".github/workflows" ]; then
  if command -v action-validator >/dev/null 2>&1; then
    echo "Running pre-commit hook for GitHub Actions: https://github.com/mpalmer/action-validator"
    scan_count=0
    for action in $(find .github/workflows -name "*.y*ml"); do
      validate="$(action-validator "$action")"
      if [ -z "$validate" ]; then
        echo "✅ $action"
      else
        echo "❌ $action"
        echo "$validate"
        exit 1
      fi
      scan_count=$((scan_count+1))
    done
    echo "action-validator scanned $scan_count GitHub Actions found no errors!"
  else
    echo "action-validator is not installed."
    echo "Install with: https://github.com/mpalmer/action-validator"
    echo "Skipping GitHub Action linting..."
  fi
else
  echo "Found no GitHub Action yaml files. Skipping action-validator linting."
fi
```

This script will run on every commit to the target repository, whether the github action yaml files are being committed, or not and prevent any commit if there are linting errors.

```
$ echo "" >> README.md && git add README.md && git commit -m "Update read-me"
Running pre-commit hook for GitHub Actions: https://github.com/mpalmer/action-validator
✅ .github/workflows/ci.yaml
✅ .github/workflows/release.yml
action-validator scanned 2 GitHub Actions found no errors!
[main c34fda3] Update read-me
 1 file changed, 2 insertions(+)

# All action-validator linting errors must be resolved before any commit will succeed.
$ echo "" >> README.md && git add README.md && git commit -m "Update read-me"
Running pre-commit hook for GitHub Actions: https://github.com/mpalmer/action-validator
Validation failed: ValidationState {
    errors: [
        Properties {
            path: "",
            detail: "Additional property 'aname' is not allowed",
        },
    ],
    missing: [],
    replacement: None,
}
❌ .github/workflows/ci.yaml
Fatal error validating .github/workflows/ci.yaml: validation failed
 ```


# Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md).


# License

Unless otherwise stated, everything in this repo is covered by the following
copyright notice:

    Copyright (C) 2021  Matt Palmer <matt@hezmatt.org>

    This program is free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License version 3, as
    published by the Free Software Foundation.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
