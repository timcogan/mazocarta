# Release Process

This project uses Semantic Versioning and publishes GitHub Releases from the version in `Cargo.toml`.

When a version bump lands on `master`, GitHub Actions creates the matching `vX.Y.Z` tag and GitHub Release automatically. The same release workflow also re-dispatches GitHub Pages so the stable site is rebuilt from that new tag.

## Versioning Rules

- MAJOR (`X.0.0`): breaking or incompatible changes.
- MINOR (`0.X.0`): backwards-compatible features.
- PATCH (`0.0.X`): backwards-compatible fixes.

## Steps To Publish

1. Update the root `Cargo.toml` version to `X.Y.Z`.
2. Run `make publish-check`.
3. Run `bash ./scripts/run-gitleaks.sh`.
4. Commit the release change.
5. Push the commit to `master`.
6. GitHub Actions will:
   - detect the version change
   - create `vX.Y.Z`
   - create the GitHub Release
   - refresh stable GitHub Pages from `vX.Y.Z`

## Retry Path

If the version bump landed but the release flow failed, rerun the `Release` workflow with `workflow_dispatch` on `master`.

That manual rerun can:

- create the missing GitHub Release when the tag or release was not finished
- re-dispatch GitHub Pages for the current released version if the release already exists

## Notes

- Automatic publishing only supports plain `X.Y.Z` versions. Suffixes like `-dev`, `-rc1`, or `+meta` are rejected by the workflow.
- Root `Cargo.toml` is the single source of truth for the release version.
- The workflow refuses to reuse a version that already has a published release.
- `scripts/publish-check.sh` validates fmt, check, clippy, tests, build, tracked-file scans, and reachable-history content scans for machine paths, secret-like values, and email addresses.
- `scripts/run-gitleaks.sh` is the history-aware secret scan and should stay clean before any public push or tag.
