#!/usr/bin/env bash
# write-homebrew-formula.sh records the three release checksums and rejects bad input.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
home_dir="$(mktemp -d)"
trap 'rm -rf "$home_dir"' EXIT

# The script writes Formula/ next to the repo root it infers from its own path.
# Run a copy inside a throwaway tree so the real formula stays untouched.
mkdir -p "${home_dir}/script" "${home_dir}/Formula"
cp "${repo_root}/script/write-homebrew-formula.sh" "${home_dir}/script/"

sha_mac="$(printf 'a%.0s' {1..64})"
sha_arm="$(printf 'b%.0s' {1..64})"
sha_amd="$(printf 'c%.0s' {1..64})"

"${home_dir}/script/write-homebrew-formula.sh" 0.1.0 "$sha_mac" "$sha_arm" "$sha_amd"
formula="${home_dir}/Formula/git-whistles.rb"
grep -F 'version "0.1.0"' "$formula" >/dev/null
grep -F "git-whistles-macos-arm64.tar.gz" "$formula" >/dev/null
grep -F "sha256 \"${sha_mac}\"" "$formula" >/dev/null
grep -F "sha256 \"${sha_arm}\"" "$formula" >/dev/null
grep -F "sha256 \"${sha_amd}\"" "$formula" >/dev/null
grep -F 'bin.install "git-whistles"' "$formula" >/dev/null

set +e
"${home_dir}/script/write-homebrew-formula.sh" 0.1.0 short "$sha_arm" "$sha_amd"
status=$?
set -e
test "$status" -ne 0

printf 'homebrew formula writer ok\n'
