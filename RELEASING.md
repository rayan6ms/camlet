# Releasing Camlet

The release process stays lightweight: deterministic local verification, Electron Builder packaging, tag-driven GitHub Releases, and manual desktop validation for the parts CI cannot prove.

## Version and Tag Conventions

- Stable releases use tags such as `v0.1.0`
- Pre-releases use tags such as `v0.2.0-beta.1`
- The Git tag must match `package.json`

## Local Verification

Run the full verification pipeline before cutting a candidate:

```bash
pnpm verify
```

Build distributable artifacts locally when needed:

```bash
pnpm package:linux
pnpm package:win
```

Packaged artifacts are written to `release/`.

## Release-Candidate Flow

Use a beta tag when you still want external desktop validation or packaging feedback. Promote to a stable tag only after the candidate has passed the manual checks in [`QA.md`](./QA.md) on the target platforms and the in-app About/diagnostics surface matches the intended artifact.

Minimum release-candidate expectations:

- at least one packaged Linux validation pass
- About/build metadata matches the intended version
- diagnostics summary is available for bug reports
- startup notices are translated and understandable
- camera fallback and retry behavior are understandable on a real desktop

## Release Checklist

1. Update `package.json` to the intended release version.
2. Run `pnpm verify`.
3. Build at least the Linux package locally with `pnpm package:linux`.
4. Open the packaged build and confirm the About card shows the expected version, channel, runtime, platform, arch, display protocol, and Electron/Chrome versions.
5. Copy the diagnostics summary once to confirm the candidate exposes usable bug-report context.
6. Run the relevant manual checks in [`QA.md`](./QA.md).
7. Review release notes or changelog content you want published.
8. Commit the release changes.
9. Create a tag such as `v0.1.0` or `v0.2.0-beta.1`.
10. Push the branch and tag to GitHub.

Example:

```bash
git tag v0.1.0
git push origin main --tags
```

## GitHub Actions

- `CI` runs typecheck, Biome, unit and integration tests, and a production build.
- `Package Artifacts` builds uploadable Linux and Windows artifacts from `main`.
- `Release` runs on version tags, builds artifacts, creates a GitHub Release, and uploads release assets.

## Expected Artifacts

- Linux: `Camlet-<version>-linux-x86_64.AppImage`
- Windows: `Camlet-<version>-windows-x64.exe`

The Windows installer remains unsigned. Code signing, notarization, auto-update, and crash-reporting services are still intentionally out of scope.

## Headless Limits

`pnpm verify` and the current deterministic suites cover:

- bootstrap contract validation
- preload API shape
- settings recovery and persistence fallback
- security and navigation policy helpers
- release metadata and diagnostics helper logic

They do not prove:

- compositor-specific transparency behavior
- camera permission prompts
- actual hardware compatibility
- always-on-top behavior after real focus changes
- packaged startup across all Linux desktop environments

Those still require real desktop validation before promoting a candidate more broadly.
