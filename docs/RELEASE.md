# Release Runbook

Step-by-step guide for publishing a qipu release. Current state is partially automated with post-release manual steps. Goal is full automation.

## Quick Reference

```bash
# 1. Ensure main is ready
git checkout main
git pull origin main

# 2. Determine next version from tags/releases (do this first)
#    - git tag --sort=-v:refname | head
#    - git ls-remote --tags origin
#    - gh release list --limit 10
#    - next = latest released/tagged patch + 1

# 3. Version bump - update all version locations (see checklist below)

# 4. Update CHANGELOG.md

# 5. Commit version bump
git add .
git commit -m "chore: bump version to X.Y.Z"

# 6. Tag and push
git tag vX.Y.Z
git push origin main
git push origin vX.Y.Z

# 7. Push tag, wait for release workflow, then update hashes/manifests (see sections below)
```

## Pre-Bump Version Check Checklist

Before editing files:

- [ ] `git tag --sort=-v:refname | head -n 20` to inspect latest local semver tags
- [ ] `git ls-remote --tags origin | grep -E 'refs/tags/v[0-9]+\.[0-9]+\.[0-9]+$'` to inspect remote tags
- [ ] `gh release list --limit 20` to inspect latest GitHub releases (published + drafts)
- [ ] Pick `X.Y.Z` from latest published/tagged release line, not from local working files

## Version Change Locations Checklist

For each release `X.Y.Z`, update all of these:

- [ ] `Cargo.toml` (`[package].version`, `qipu-core` dependency version, `[package.metadata.rpm].version`)
- [ ] `crates/qipu-core/Cargo.toml` (`[package].version`)
- [ ] `Cargo.lock` (`qipu` + `qipu-core` package entries; regenerate via `cargo build`/`cargo test`)
- [ ] `CHANGELOG.md` (new release heading/date)
- [ ] `tests/golden/version.txt`
- [ ] `distribution/homebrew-qipu/Formula/qipu.rb` (version, URLs, SHA256)
- [ ] `distribution/aur/PKGBUILD` (`pkgver`, sources, SHA256)
- [ ] `distribution/aur/.SRCINFO` (regenerate from `PKGBUILD`)
- [ ] `distribution/scoop/qipu.json` (version, URL, hash)
- [ ] `distribution/winget/manifests/m/mwaldstein/qipu/X.Y.Z/mwaldstein.qipu.installer.yaml`
- [ ] `distribution/winget/manifests/m/mwaldstein/qipu/X.Y.Z/mwaldstein.qipu.locale.en-US.yaml`
- [ ] `distribution/winget/manifests/m/mwaldstein/qipu/X.Y.Z/mwaldstein.qipu.yaml`

Validation commands:

```bash
# Ensure target version appears where expected
grep -R "X.Y.Z" Cargo.toml crates/qipu-core/Cargo.toml CHANGELOG.md tests/golden/version.txt distribution

# Ensure prior release version does not remain in active manifests
grep -R "OLD.X.Y" distribution/homebrew-qipu distribution/aur distribution/scoop

# Winget keeps historical dirs. Confirm new one exists:
ls distribution/winget/manifests/m/mwaldstein/qipu/X.Y.Z
```

## Automated Steps (GitHub Actions)

The release workflow (`.github/workflows/release.yml`) handles:

| Step | Status | Notes |
|------|--------|-------|
| Build binaries (all platforms) | ✅ Automated | macOS x86_64/aarch64, Linux glibc x86_64/aarch64, Linux musl x86_64/aarch64, Windows x86_64 |
| Create GitHub release | ✅ Automated | Attaches all binaries |
| Generate SHA256SUMS | ✅ Automated | Combined checksums file |
| Sign binaries (cosign) | ✅ Automated | Uses Sigstore OIDC |
| Publish to crates.io | ✅ Automated | Both qipu-core and qipu crates |
| Build .deb packages | ✅ Automated | Both x86_64 and aarch64 |
| Build .rpm package | ✅ Automated | x86_64 only |

**Trigger:** Push a tag matching `v*`

**Monitor:** https://github.com/mwaldstein/qipu/actions

## Post-Release Manual Steps

After the GitHub release is published, update external package managers:

### 0. Manifest update checklist (required)

Update every in-repo distribution manifest for `X.Y.Z`:

- `distribution/homebrew-qipu/Formula/qipu.rb`
- `distribution/aur/PKGBUILD`
- `distribution/aur/.SRCINFO`
- `distribution/scoop/qipu.json`
- `distribution/winget/manifests/m/mwaldstein/qipu/X.Y.Z/mwaldstein.qipu.installer.yaml`
- `distribution/winget/manifests/m/mwaldstein/qipu/X.Y.Z/mwaldstein.qipu.locale.en-US.yaml`
- `distribution/winget/manifests/m/mwaldstein/qipu/X.Y.Z/mwaldstein.qipu.yaml`

Use `SHA256SUMS` from the GitHub release as source of truth.

### 1. Homebrew (macOS/Linux)

**Status:** ✅ Formula ready, ❌ Tap repo not created

**Current State:**
- Formula exists: `distribution/homebrew-qipu/Formula/qipu.rb`
- Need to create repo: `mwaldstein/homebrew-qipu`
- Uses release binary download (not source build)

**After release:**
```bash
cd distribution/homebrew-qipu
# Edit Formula/qipu.rb:
#   - version
#   - Intel URL + SHA256 (x86_64-apple-darwin)
#   - Apple Silicon URL + SHA256 (aarch64-apple-darwin)
# Get hashes from release:
#   curl -sL "https://github.com/mwaldstein/qipu/releases/download/vX.Y.Z/SHA256SUMS" | grep apple-darwin
git add .
git commit -m "qipu X.Y.Z"
git push origin main
```

**Infrastructure needed:** Create `mwaldstein/homebrew-qipu` repo on GitHub, then run `distribution/setup-homebrew-tap.sh`.

### 2. AUR (Arch Linux)

**Status:** ⚠️ Manual updates required

**Files:** `distribution/aur/PKGBUILD`

**After release:**
```bash
cd distribution/aur
# Update PKGBUILD:
#   - pkgver=X.Y.Z
#   - pkgrel=1 (reset to 1 on version change)
#   - keep source URLs on -unknown-linux-gnu (Arch uses glibc)
#   - sha256sums_x86_64=$(curl -sL https://github.com/mwaldstein/qipu/releases/download/vX.Y.Z/SHA256SUMS | grep x86_64-unknown-linux-gnu | cut -d' ' -f1)
#   - sha256sums_aarch64=$(curl -sL https://github.com/mwaldstein/qipu/releases/download/vX.Y.Z/SHA256SUMS | grep aarch64-unknown-linux-gnu | cut -d' ' -f1)

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Submit to AUR (requires AUR account and git setup)
cd /tmp
git clone ssh://aur@aur.archlinux.org/qipu-bin.git aur-qipu-bin
cd aur-qipu-bin
cp /path/to/qipu/distribution/aur/PKGBUILD .
cp /path/to/qipu/distribution/aur/.SRCINFO .
git add .
git commit -m "Update to vX.Y.Z"
git push origin master
```

**User install:** `yay -S qipu-bin` or `paru -S qipu-bin`

### 3. Scoop (Windows)

**Status:** ⚠️ Manual updates required

**Files:** `distribution/scoop/qipu.json`

**After release:**
```bash
# Download Windows release and get SHA256
curl -sL -o qipu.zip "https://github.com/mwaldstein/qipu/releases/download/vX.Y.Z/qipu-X.Y.Z-x86_64-pc-windows-msvc.zip"
shasum -a 256 qipu.zip
rm qipu.zip

# Edit distribution/scoop/qipu.json:
#   - Update "version" to "X.Y.Z"
#   - Update "url" to new release URL
#   - Update "hash" to the SHA256

git add distribution/scoop/qipu.json
git commit -m "chore: update scoop manifest to vX.Y.Z"
```

**User install:**
```powershell
scoop install https://raw.githubusercontent.com/mwaldstein/qipu/main/distribution/scoop/qipu.json
```

**Future:** Create `mwaldstein/scoop-qipu` bucket for `scoop bucket add qipu`.

### 4. Winget (Windows)

**Status:** ⚠️ Manual PR to microsoft/winget-pkgs required

**Files:** `distribution/winget/manifests/m/mwaldstein/qipu/X.Y.Z/`

**After release:**
```bash
VERSION=X.Y.Z
mkdir -p distribution/winget/manifests/m/mwaldstein/qipu/${VERSION}

# Create three manifest files (see existing 0.3.27 for template):
#   - mwaldstein.qipu.installer.yaml
#   - mwaldstein.qipu.locale.en-US.yaml  
#   - mwaldstein.qipu.yaml

# Get SHA256 for Windows zip
curl -sL "https://github.com/mwaldstein/qipu/releases/download/v${VERSION}/SHA256SUMS" | grep windows

# Or use wingetcreate:
wingetcreate update mwaldstein.qipu --version ${VERSION} \
  --urls "https://github.com/mwaldstein/qipu/releases/download/v${VERSION}/qipu-${VERSION}-x86_64-pc-windows-msvc.zip|x64"

# Submit to winget-pkgs:
# 1. Fork https://github.com/microsoft/winget-pkgs
# 2. Copy distribution/winget/manifests/m/mwaldstein/qipu/X.Y.Z/ to your fork
# 3. Submit PR
```

**User install:** `winget install mwaldstein.qipu`

### 5. Nix

**Status:** ❌ Not implemented

Issue `qipu-u0g` tracks Nix support. Requires creating a flake.nix.

## Automation Wishlist

Future improvements to reduce manual work:

| Task | Priority | Approach |
|------|----------|----------|
| Auto-update AUR | P2 | GitHub Action that clones aur repo, updates, pushes |
| Auto-update Scoop | P2 | GitHub Action that edits JSON, commits |
| Auto-update Winget | P2 | Use `wingetcreate` in GitHub Action + bot PR |
| Auto-update Homebrew tap | P3 | GitHub Action in tap repo watches releases |
| Version bump automation | P3 | Script to update all Cargo.toml versions |
| Changelog generation | P3 | Conventional commits → CHANGELOG.md |

## Verification Checklist

After each release, verify:

- [ ] GitHub release page shows all binaries + SHA256SUMS
- [ ] `cargo install qipu` works (wait for crates.io propagation)
- [ ] Install scripts work:
  - [ ] `curl -fsSL https://.../install.sh | bash` (Linux/macOS)
  - [ ] `irm https://.../install.ps1 | iex` (Windows)
- [ ] AUR updated (if applicable)
- [ ] Scoop manifest updated (if applicable)
- [ ] Winget PR submitted (if applicable)
- [ ] Homebrew tap updated (once created)

## Emergency Rollback

If a release is broken:

```bash
# Delete tag (prevents further triggers)
git push --delete origin vX.Y.Z
git tag -d vX.Y.Z

# Yank from crates.io (if needed)
cargo yank -p qipu --version X.Y.Z
cargo yank -p qipu-core --version X.Y.Z

# Delete GitHub release manually via web UI
```

## References

- [distribution.md](../specs/distribution.md) - Specification
- [distribution/README.md](../distribution/README.md) - Package manager details
- `.github/workflows/release.yml` - Automation
