#!/usr/bin/env bash
# Installs `dx` at exactly the dioxus version this project resolves to.
# dx refuses to build when its version differs from the dioxus crate version.
set -euo pipefail

cd "$(dirname "$0")/.."
version="$(cargo pkgid dioxus | awk -F'@' '{print $NF}')"
echo "Installing dioxus-cli ${version} (matching the resolved dioxus crate)…"
cargo install dioxus-cli --locked --version "${version}"
