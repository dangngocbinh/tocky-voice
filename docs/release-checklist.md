# Release checklist

Manual steps a maintainer runs around a tag. `.github/workflows/release.yml` handles the
build/sign/draft; everything below is what a human still has to do.

## One-time: Apple signing and notarisation

Until these secrets exist the pipeline builds unsigned, and the app knows it — on an
unsigned macOS build `code_signature.rs` reports "cannot self-install" and the update
button sends people to the download page instead. Nothing has to be switched over on the
day the certificate arrives; the next tagged build signs itself and the app notices.

1. **Certificate.** Apple Developer → Certificates → **Developer ID Application**
   (Keychain Access → Certificate Assistant makes the CSR). Download it and open it, so
   it lands in the login keychain together with its private key.
2. **Export it** from Keychain Access as a `.p12` with a password, then:
   ```sh
   base64 -i certificate.p12 | pbcopy
   ```
3. **Signing identity name** — the exact string to put in the secret:
   ```sh
   security find-identity -v -p codesigning
   ```
   Use the full `Developer ID Application: Name (TEAMID)`, quotes excluded.
4. **App-specific password** for notarisation: appleid.apple.com → Sign-In and Security →
   App-Specific Passwords. The account password itself will not work.
5. Add all six as GitHub repository secrets (Settings → Secrets and variables → Actions):

   | Secret | Value |
   | --- | --- |
   | `APPLE_CERTIFICATE` | base64 of the `.p12` from step 2 |
   | `APPLE_CERTIFICATE_PASSWORD` | the password used in that export |
   | `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Name (TEAMID)` |
   | `APPLE_ID` | the Apple account email |
   | `APPLE_PASSWORD` | the app-specific password from step 4 |
   | `APPLE_TEAM_ID` | 10-character team ID from the membership page |

   `APPLE_SIGNING_IDENTITY` is also the switch the workflow reads: set, and the release
   notes footer changes to the signed wording and the build is verified with `codesign`
   and `xcrun stapler` before it can reach the draft.

6. On the **first signed release**, say in the changelog that anyone updating from an
   unsigned build has to grant Accessibility once more — macOS keys that permission to
   the code signature, and this is the release where the identity changes. It happens
   exactly once. Update the Gatekeeper sections of `README.md` / `README.en.md` at the
   same time, since right-click → Open and `xattr -cr` stop being necessary.

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
2. Sanity-check the release notes read correctly (they came from `CHANGELOG.md`). They
   are rendered as Markdown inside the app's "what's new" panel, so headings and bullets
   there should look like headings and bullets, not like `###` and `-`.
3. On macOS, confirm the build really is signed — the `Verify the macOS bundle` step in
   the workflow does this, but it only runs when `APPLE_SIGNING_IDENTITY` is set:
   ```sh
   codesign -dv --verbose=2 "/Applications/Tocky Voice.app"   # after installing the .dmg
   ```
   `Authority=Developer ID Application: …` is what makes in-app updates work. Without it
   the release still functions, but every macOS user updates by hand.
4. Click **Publish**.

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
