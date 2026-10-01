#!/usr/bin/env bash
# Rewrite Formula/git-whistles.rb so Homebrew downloads one published release.
# Usage: write-homebrew-formula.sh VERSION MACOS_ARM64_SHA LINUX_ARM64_SHA LINUX_AMD64_SHA
set -euo pipefail

if [ "$#" -ne 4 ]; then
  printf 'usage: %s VERSION MACOS_ARM64_SHA LINUX_ARM64_SHA LINUX_AMD64_SHA\n' "$0" >&2
  exit 1
fi

version="$1"
macos_arm64="$2"
linux_arm64="$3"
linux_amd64="$4"

require_sha() {
  if ! printf '%s' "$1" | grep -Eq '^[0-9a-f]{64}$'; then
    printf 'error: sha256 must be 64 hex characters: %s\n' "$1" >&2
    exit 1
  fi
}

require_sha "$macos_arm64"
require_sha "$linux_arm64"
require_sha "$linux_amd64"

if ! printf '%s' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  printf 'error: version must look like 0.1.0: %s\n' "$version" >&2
  exit 1
fi

root="$(cd "$(dirname "$0")/.." && pwd)"
formula="${root}/Formula/git-whistles.rb"
tmp="$(mktemp)"

cat >"$tmp" <<EOF
# frozen_string_literal: true

# Install via the repo script (binary, shims, and the wt alias):
#   curl --proto '=https' --tlsv1.2 -fsSL \\
#     https://raw.githubusercontent.com/mezis/git-whistles/master/install.sh | bash
#
# Stable install downloads the release tarball. Intel Macs are not published.
#   brew tap mezis/git-whistles https://github.com/mezis/git-whistles
#   brew install mezis/git-whistles/git-whistles
class GitWhistles < Formula
  desc "Helpers for classic Git workflows (chop, ff-all-branches, list-branches, worktree-list, stash-and-checkout, staging, merge-po, changes, shim)"
  homepage "https://github.com/mezis/git-whistles"
  license "MIT"
  version "${version}"

  on_macos do
    on_arm do
      url "https://github.com/mezis/git-whistles/releases/download/v${version}/git-whistles-macos-arm64.tar.gz"
      sha256 "${macos_arm64}"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/mezis/git-whistles/releases/download/v${version}/git-whistles-linux-amd64.tar.gz"
      sha256 "${linux_amd64}"
    end
    on_arm do
      url "https://github.com/mezis/git-whistles/releases/download/v${version}/git-whistles-linux-arm64.tar.gz"
      sha256 "${linux_arm64}"
    end
  end

  # brew install --HEAD still builds master. Rust is only required for that.
  head "https://github.com/mezis/git-whistles.git", branch: "master"

  depends_on "rust" => :build if build.head?

  def install
    if build.head?
      system "cargo", "install", *std_cargo_args
    else
      bin.install "git-whistles"
    end
  end

  test do
    assert_match "Helpers for classic Git", shell_output("#{bin}/git-whistles --help")
  end
end
EOF

mv "$tmp" "$formula"
