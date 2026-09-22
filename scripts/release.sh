#!/usr/bin/env bash
# Bumps the crate version in Cargo.toml and Cargo.lock for a release.
# release.yml refuses to build a tag whose name disagrees with the Cargo.toml
# version (the footer shows CARGO_PKG_VERSION, so a mismatch would ship a binary
# labelled with the wrong version). Bumping by hand is what let the two drift on
# v0.1.1, which failed the release build.
# Edits the working tree only — commit and tag yourself.
set -euo pipefail

cd "$(dirname "$0")/.."

if [ $# -ne 1 ]; then
  echo "usage: $0 X.Y.Z    (bare semver, no leading 'v')" >&2
  exit 1
fi

version="$1"
if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "error: '$version' is not a bare X.Y.Z version (no leading 'v', no suffix)" >&2
  exit 1
fi
tag="v${version}"

# A tag that already exists means this version is spoken for: pushing it again
# can't trigger a release, and reusing the number ships two different binaries
# under one version.
if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
  echo "error: tag ${tag} already exists locally" >&2
  exit 1
fi
if [ -n "$(git ls-remote --tags origin "refs/tags/${tag}" 2>/dev/null)" ]; then
  echo "error: tag ${tag} already exists on origin" >&2
  exit 1
fi

# Only the first `version = ` line: the rest of Cargo.toml is dependency versions.
current="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
echo "Bumping ${current} -> ${version}…"
perl -i -pe "\$done ||= s/^version = \"[^\"]*\"/version = \"${version}\"/ unless \$done" Cargo.toml

if [ "$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)" != "${version}" ]; then
  echo "error: failed to rewrite the version in Cargo.toml" >&2
  exit 1
fi

# Cargo.lock records chauve's own version too; leaving it stale breaks --locked builds.
cargo update --workspace --offline >/dev/null

echo "Cargo.toml and Cargo.lock now read ${version}. To release:"
echo "  git commit -am 'release ${tag}'"
echo "  git tag ${tag}"
echo "  git push origin HEAD ${tag}   # pushing the tag triggers the release workflow"
