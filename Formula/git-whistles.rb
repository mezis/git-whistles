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
  version "2.0.0"

  on_macos do
    on_arm do
      url "https://github.com/mezis/git-whistles/releases/download/v2.0.0/git-whistles-macos-arm64.tar.gz"
      sha256 "9204974caf1f4d635d5f364c54a4045d115b281adae2694c1644f45b693b48e4"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/mezis/git-whistles/releases/download/v2.0.0/git-whistles-linux-amd64.tar.gz"
      sha256 "75aaa95643b1b93c39df5f2c0da12f4750c8bf2932e4264e7eff3e046b9a160d"
    end
    on_arm do
      url "https://github.com/mezis/git-whistles/releases/download/v2.0.0/git-whistles-linux-arm64.tar.gz"
      sha256 "0edb521bfe189f704b5cf8a98fbb83bba377497a5752ec3b8725f21b3beaf382"
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
