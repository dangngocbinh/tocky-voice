# Release checklist

Manual steps a maintainer runs around a tag. `.github/workflows/release.yml` handles the
build/sign/draft; everything below is what a human still has to do.

## Before tagging

1. Bump the version in all three places — a mismatch fails CI's guard step, but catching
   it before pushing the tag saves a wasted build:
   - `src-tauri/tauri.conf.json` → `"version"`
   - `src-tauri/Cargo.toml` → `[package].version`
   - `package.json` → `"version"`
2. Add a `## [x.y.z] - YYYY-MM-DD` section to `CHANGELOG.md`, written for a user — it
   becomes the GitHub release body and the in-app "what's new" text verbatim.
3. Open a PR with the version bump + changelog entry, get it reviewed, merge to `main`.

## Tag and wait

```sh
git tag vX.Y.Z && git push origin vX.Y.Z
```

Watch the `Release` workflow. It runs `create-release` → four `build` jobs (macOS
arm64/x64, Windows, Linux) → `verify-manifest`. Any version mismatch or incomplete
`latest.json` fails the run before anything reaches users — read the failing step's
error, it names exactly what's wrong.

## Review the draft

1. Open the draft release on GitHub. Confirm all expected assets are present:
   installers (`.dmg` ×2, `.exe`, `.msi`, `.AppImage`, `.deb`, `.rpm`) **and** the
   updater artifacts (`.app.tar.gz` ×2, `.nsis.zip`, `.AppImage.tar.gz`) **and**
   `latest.json`.
2. Sanity-check the release notes read correctly (they came from `CHANGELOG.md`).
3. Click **Publish**.

## After publishing

`releases/latest/download/latest.json` only resolves once the release is published —
that's what gates the update from reaching anyone before this point.

```sh
curl -s https://github.com/dangngocbinh/tocky-voice/releases/latest/download/latest.json \
  | python3 -c "import json,sys; d=json.load(sys.stdin); print(d['version'], list(d['platforms'].keys()))"
```

Expect the new version and all four platform keys. Then run an install-and-update smoke
test from the previous version on at least one platform before considering the release
done.

## Key material

The Ed25519 update-signing keypair lives outside this repo. The private key and its
password are in the password manager (entry: "Tocky Voice — Tauri updater key"), and the
same two values are the `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
repo secrets. Losing the private key means no existing install can ever be offered an
update again — there is no recovery but telling every user to reinstall by hand.
