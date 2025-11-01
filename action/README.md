# action-validator

This composite action downloads `action-validator` via the `gh` cli, caches the binary using [`actions/cache`](https://github.com/actions/cache)<sup>1</sup> and runs it in verbose mode for better log output. Relevant files are discovered automatically (both `.yml` and `.yaml` are supported) and it only feeds files tracked by Git to the validator. All Linux and macOS runners are supported<sup>2</sup>, because prebuilt binaries are released for those.

The action also has a few optional inputs:
- `version` -- by default `0.8.0`. You may need to define it yourself to stay up-to-date, especially if you have locked the action to a SHA as you should.
- `install-path` -- by default `/usr/local/bin`. The path may not work on some `self-hosted` runners or in jobs that are containerised, so in those cases switch it to one that is available in the `PATH` variable of the runner environment.

<sup>1</sup> to ensure effective caching with [cache scopes](https://github.com/actions/cache/blob/0057852bfaa89a56745cba8c7296529d2fc39830/README.md#cache-scopes) please make sure the action also runs on the default branch.  
<sup>2</sup> support for Apple Silicon macOS runners started in version `0.2.0` and for arm Linux runners in version `0.5.2`.

## Usage example

```yml
name: Validate
on:
  pull_request:
  push:
    branches:
      - main
      - master

jobs:
  action-validator:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v5

      - name: action-validator
        uses: mpalmer/action-validator/action@main # please lock to the latest SHA for secure use
```
