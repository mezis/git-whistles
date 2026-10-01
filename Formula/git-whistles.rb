# frozen_string_literal: true

# Install via the repo script (binary, shims, and the wt alias):
#   curl --proto '=https' --tlsv1.2 -fsSL \
#     https://raw.githubusercontent.com/mezis/git-whistles/master/install.sh | bash
#
# A version tag publishes release tarballs. That workflow rewrites this file
# with their checksums. Until then, Homebrew builds master.
#   brew tap mezis/git-whistles https://github.com/mezis/git-whistles
#   brew install mezis/git-whistles/git-whistles
class GitWhistles < Formula
  desc "Helpers for classic Git workflows (chop, ff-all-branches, list-branches, worktree-list, stash-and-checkout, staging, merge-po, changes, shim)"
  homepage "https://github.com/mezis/git-whistles"
  license "MIT"
  head "https://github.com/mezis/git-whistles.git", branch: "master"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "Helpers for classic Git", shell_output("#{bin}/git-whistles --help")
  end
end
