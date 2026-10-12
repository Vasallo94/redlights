# Security

## Reporting a vulnerability

Please do not report security problems in public issues. Use GitHub's private reporting instead: go to the [Security tab](https://github.com/Vasallo94/redlights/security) and click **Report a vulnerability**. Only the maintainer can see the report.

You can expect an answer within a week. Fixes are released as a new version and credited to the reporter unless they prefer to stay anonymous.

## Supported versions

Only the [latest release](https://github.com/Vasallo94/redlights/releases/latest) receives fixes.

## What RedLights can access

RedLights only changes the colors the display outputs. It does not read the screen, record input, use the network or store data beyond the Mac app's brightness setting and its login item. It needs no special permissions on any platform.

## Verifying downloads

Every release file is built by GitHub Actions from a tagged commit of this repository and published with:

- a `.sha256` checksum file:

  ```bash
  shasum -a 256 -c redlights-linux-x86_64.tar.gz.sha256
  ```

- a [GitHub artifact attestation](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations), which proves the file was built by this repository's release workflow from a specific tag:

  ```bash
  gh attestation verify redlights-linux-x86_64.tar.gz --repo Vasallo94/redlights
  ```

## Code signing

The release binaries are not signed with a commercial certificate yet:

- **macOS**: the app is ad-hoc signed but not notarized, so Gatekeeper blocks it the first time you open a downloaded copy. Installing with Homebrew avoids the warning.
- **Windows**: the executable is unsigned, so SmartScreen may warn you the first time.
- **Linux**: binaries are distributed as plain archives; integrity is covered by the checksums and attestations above.

The attestation is the strongest guarantee available today: it ties each file to the public source and build log that produced it.
