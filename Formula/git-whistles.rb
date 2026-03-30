# frozen_string_literal: true

# Install: brew tap mezis/git-whistles && brew install git-whistles
# (Tap is this repo; Formula lives in Formula/)
class GitWhistles < Formula
  desc "Helpers for classic Git workflows (chop, ff-all-branches, list-branches, stash-and-checkout, staging, merge-po, changes, shim)"
  homepage "https://github.com/mezis/git-whistles"
  version "0.1.0"

  if OS.mac? && Hardware::CPU.arm?
    url "https://github.com/mezis/git-whistles/releases/download/v#{version}/git-whistles-macos-arm64.tar.gz"
    sha256 "placeholder" # Run: shasum -a 256 < tarball; update on release
  elsif OS.linux? && Hardware::CPU.arm?
    url "https://github.com/mezis/git-whistles/releases/download/v#{version}/git-whistles-linux-arm64.tar.gz"
    sha256 "placeholder"
  elsif OS.linux? && Hardware::CPU.intel?
    url "https://github.com/mezis/git-whistles/releases/download/v#{version}/git-whistles-linux-amd64.tar.gz"
    sha256 "placeholder"
  else
    url "https://github.com/mezis/git-whistles/archive/refs/tags/v#{version}.tar.gz"
    sha256 "placeholder"
    depends_on "rust" => :build
  end

  def install
    if File.exist?("git-whistles") && File.file?("git-whistles")
      bin.install "git-whistles"
    else
      system "cargo", "install", *std_cargo_args(path: ".")
    end
  end

  test do
    assert_match "Helpers for classic Git", shell_output("#{bin}/git-whistles --help")
  end
end
