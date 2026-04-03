# Publishing to a Launchpad PPA

This documents the steps taken to publish `mdview` to the PPA `ppa:hopung/tools` on Ubuntu 22.04 (jammy).

## One-time setup

### 1. Install packaging tools

```
sudo apt install devscripts debhelper dh-make dput
```

### 2. Create a GPG key

```
gpg --full-generate-key
```

- Key type: RSA and RSA
- Key size: 4096
- Expiry: 0 (no expiry)
- Use the same email as your Launchpad account

### 3. Upload key to Ubuntu keyserver

```
gpg --keyserver keyserver.ubuntu.com --send-keys <FINGERPRINT>
```

Find your fingerprint with `gpg --list-keys --keyid-format long`.

### 4. Link key to Launchpad

Go to https://launchpad.net/~/+editpgpkeys, paste the fingerprint, and follow the email confirmation.

### 5. Create a PPA

Go to https://launchpad.net/~/+activate-ppa and create one (e.g., `tools`).

## Per-release workflow

### 1. Vendor Rust dependencies

Launchpad builders have no internet access to crates.io, so all crate dependencies must be included in the source package.

```
cargo vendor
```

Create `.cargo/config.toml`:

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
```

Verify it builds offline:

```
cargo build --release --frozen
```

### 2. Debian packaging files

All files live in `debian/`:

- `debian/control` — package metadata and build dependencies
- `debian/rules` — build script (calls `cargo build --release --frozen`)
- `debian/changelog` — version, target distro (e.g., `jammy`), and changes
- `debian/copyright` — license info for the package and vendored crates
- `debian/source/format` — set to `3.0 (quilt)`
- `debian/source/options` — `extend-diff-ignore` patterns to exclude build artifacts

### 3. Create the orig tarball

The tarball must:
- Contain a top-level directory named `<package>-<version>` (e.g., `mdview-0.1.0`)
- Exclude `debian/`, `.git/`, build artifacts, and any local-only files
- Include the `vendor/` directory

```
cd /home/chs/Work
tar czf mdview_0.1.0.orig.tar.gz \
  --exclude='MDViewer/.git' \
  --exclude='MDViewer/debian' \
  --exclude='MDViewer/target' \
  --exclude='MDViewer/parts' \
  --exclude='MDViewer/prime' \
  --exclude='MDViewer/stage' \
  --exclude='MDViewer/.claude' \
  --exclude='*.snap' \
  --transform='s,^MDViewer,mdview-0.1.0,' \
  MDViewer
```

**Important:** Use full paths in `--exclude` (e.g., `MDViewer/target` not just `target`), otherwise tar will also exclude matching subdirectories like `vendor/cc/src/target/`.

### 4. Build the signed source package

```
debuild -S -sa -d -k<FINGERPRINT>
```

Flags:
- `-S` — source-only build (no local binary build)
- `-sa` — include the orig tarball
- `-d` — skip local build-dependency check (needed if Rust is from rustup, not apt)
- `-k<FINGERPRINT>` — GPG key to sign with

### 5. Upload to PPA

```
dput ppa:hopung/tools ../mdview_0.1.0-1_source.changes
```

Launchpad will email you when the build succeeds or fails. Monitor at:
https://launchpad.net/~hopung/+archive/ubuntu/tools

### 6. Install from PPA

```
sudo add-apt-repository ppa:hopung/tools
sudo apt install mdview
```

## Updating the package

1. Update the code
2. Re-vendor if dependencies changed: `cargo vendor`
3. Update `debian/changelog` (bump version, e.g., `0.1.1-1`)
4. Recreate the orig tarball with the new version
5. `debuild -S -sa -d -k<FINGERPRINT>`
6. `dput ppa:hopung/tools ../mdview_<version>_source.changes`

## Gotchas

- **Vendoring is required.** Launchpad builders cannot download from crates.io.
- **`--exclude` in tar is greedy.** `--exclude='target'` matches at any depth. Use `--exclude='MDViewer/target'` to only exclude the top-level build directory.
- **`extend-diff-ignore` in `debian/source/options`** prevents `dpkg-source` from complaining about local files (snap artifacts, `.claude/`, etc.) that aren't in the orig tarball.
- **GPG key must be on the Ubuntu keyserver**, not just your local keyring. Launchpad verifies the signature on upload.
- **One PPA serves multiple packages.** You don't need a new PPA per project.
