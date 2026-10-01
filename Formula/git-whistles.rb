# frozen_string_literal: true

# Install via the repo script (binary, shims, and the wt alias):
#   curl --proto '=https' --tlsv1.2 -fsSL \
#     https://raw.githubusercontent.com/mezis/git-whistles/master/install.sh | bash
#
# Stable install downloads the release tarball. Intel Macs are not published.
#   brew tap mezis/git-whistles https://github.com/mezis/git-whistles
#   brew install mezis/git-whistles/git-whistles
class GitWhistles < Formula
  desc "Helpers for classic Git workflows (chop, ff-all-branches, list-branches, worktree-list, stash-and-checkout, staging, merge-po, changes, shim)"
  homepage "https://github.com/mezis/git-whistles"
  license "MIT"
  version "2.0.1"

  on_macos do
    on_arm do
      url "https://github.com/mezis/git-whistles/releases/download/v2.0.1/git-whistles-macos-arm64.tar.gz"
      sha256 "e056d27e43ea993b5c88013e656493f92316ff142652f4af896bf8c1abe3667e"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/mezis/git-whistles/releases/download/v2.0.1/git-whistles-linux-amd64.tar.gz"
      sha256 "3c63a6f64fe11ad7dd9738955517f1f141e953cb58b5f9de87175346cf9caee5"
    end
    on_arm do
      url "https://github.com/mezis/git-whistles/releases/download/v2.0.1/git-whistles-linux-arm64.tar.gz"
      sha256 "53082a8893fb2aa2ade6e22f61eb7deb4928f6a35ea2754c64797e6869975e62"
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
