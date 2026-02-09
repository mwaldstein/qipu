# Distribution Manifests

This directory contains package manager manifests for distributing qipu.

## Winget

Location: `distribution/winget/manifests/m/mwaldstein/qipu/`

Structure follows the [Winget package manifest schema](https://aka.ms/winget-manifest.schema.json).

### Submitting to Winget

After a new release is published:

1. Update the manifests in `distribution/winget/` with the new version and SHA256 hashes
2. Fork [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs)
3. Copy the version directory to `manifests/m/mwaldstein/qipu/`
4. Submit a PR to winget-pkgs

Or use wingetcreate:

```powershell
wingetcreate update mwaldstein.qipu --version 0.3.19 --urls "https://github.com/mwaldstein/qipu/releases/download/v0.3.19/qipu-0.3.19-x86_64-pc-windows-msvc.zip|x64"
```

### Manual Installation via Winget

Once published:

```powershell
winget install mwaldstein.qipu
```

## Scoop

Location: `distribution/scoop/qipu.json`

### Setting up Scoop Bucket

For now, users can install directly from the manifest URL:

```powershell
scoop install https://raw.githubusercontent.com/mwaldstein/qipu/main/distribution/scoop/qipu.json
```

### Future: Dedicated Bucket

Consider creating `mwaldstein/scoop-qipu` bucket for easier installation:

```powershell
scoop bucket add qipu https://github.com/mwaldstein/scoop-qipu
scoop install qipu
```

### Updating SHA256 Hashes

The manifests contain placeholder SHA256 hashes (`0000000000...`). These must be updated with actual release hashes before submission.

To get the correct hash after a release:

```powershell
# Download the release
Invoke-WebRequest -Uri "https://github.com/mwaldstein/qipu/releases/download/v0.3.19/qipu-0.3.19-x86_64-pc-windows-msvc.zip" -OutFile qipu.zip

# Calculate SHA256
Get-FileHash -Path qipu.zip -Algorithm SHA256
```

Or use the SHA256SUMS file from the release.

## AUR (Arch Linux)

Location: `distribution/aur/PKGBUILD`

### Installation

Install using your favorite AUR helper:

```bash
# Using yay
yay -S qipu-bin

# Using paru
paru -S qipu-bin

# Manual installation
git clone https://aur.archlinux.org/qipu-bin.git
cd qipu-bin
makepkg -si
```

### Submitting to AUR

After a new release is published:

1. Update `pkgver` in `distribution/aur/PKGBUILD` to the new version
2. Update `sha256sums_x86_64` and `sha256sums_aarch64` with actual hashes from the release
3. Reset `pkgrel=1` if `pkgver` changed, otherwise increment
4. Regenerate `.SRCINFO`: `makepkg --printsrcinfo > .SRCINFO`
5. Commit and push to the AUR git repository

To update hashes from the release SHA256SUMS file:

```bash
# Download release artifacts
curl -sL "https://github.com/mwaldstein/qipu/releases/download/v0.3.19/SHA256SUMS" | grep "x86_64-unknown-linux-gnu"
curl -sL "https://github.com/mwaldstein/qipu/releases/download/v0.3.19/SHA256SUMS" | grep "aarch64-unknown-linux-gnu"
```

**Note**: The AUR package is `qipu-bin` since it uses pre-built binaries. A source-only `qipu` package could be added in the future for those who prefer building from source.

## Nix

Status: Future work. See issue `qipu-u0g`.
