#!/usr/bin/env bash
# Install git-whistles, git-* shims, and the wt alias for bash and zsh.
#
#   curl --proto '=https' --tlsv1.2 -fsSL \
#     https://raw.githubusercontent.com/mezis/git-whistles/master/install.sh | bash
#
# Safe to re-run. curl | bash installs the published release with Homebrew when
# brew is available, otherwise builds with cargo (installing Rust via Homebrew
# or rustup). ./install.sh inside a clone builds that clone.

set -euo pipefail

GIT_WHISTLES_GIT_URL="${GIT_WHISTLES_GIT_URL:-https://github.com/mezis/git-whistles.git}"
GIT_WHISTLES_GIT_BRANCH="${GIT_WHISTLES_GIT_BRANCH:-master}"
# auto | brew | cargo. auto: this clone if the script is a file, else Homebrew, else cargo.
GIT_WHISTLES_INSTALL_FROM="${GIT_WHISTLES_INSTALL_FROM:-auto}"

SHIM_DIR="${HOME:?HOME is not set}/.local/bin"
STABLE_BIN="${SHIM_DIR}/git-whistles"
ENV_FILE="${HOME}/.config/git-whistles/env.sh"
MARKER_BEGIN="# >>> git-whistles initialize >>>"
MARKER_END="# <<< git-whistles initialize <<<"

info() {
  printf '==> %s\n' "$*" >&2
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

assert_supported_os() {
  case "$1" in
    Darwin | Linux) ;;
    *) die "git-whistles supports macOS and Linux (this machine reports $1)" ;;
  esac
}

# Homebrew is optional. Standard prefixes are checked because a non-interactive
# curl | bash does not see shellenv from the user's profile.
locate_brew() {
  if command -v brew >/dev/null 2>&1; then
    return 0
  fi
  local candidate
  for candidate in \
    /opt/homebrew/bin/brew \
    /usr/local/bin/brew \
    "${HOME}/.linuxbrew/bin/brew" \
    /home/linuxbrew/.linuxbrew/bin/brew
  do
    if [ -x "$candidate" ]; then
      eval "$("$candidate" shellenv)"
      return 0
    fi
  done
  return 1
}

resolve_path() {
  if command -v realpath >/dev/null 2>&1; then
    realpath "$1"
    return
  fi
  if command -v python3 >/dev/null 2>&1; then
    python3 -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$1"
    return
  fi
  perl -MCwd=realpath -e 'print realpath($ARGV[0]), "\n"' "$1"
}

ensure_git() {
  if command -v git >/dev/null 2>&1; then
    return 0
  fi
  if command -v brew >/dev/null 2>&1; then
    info "Installing git with Homebrew"
    brew install git
    return 0
  fi
  die "git is required to build git-whistles. Install git and run this script again."
}

ensure_cargo() {
  if command -v cargo >/dev/null 2>&1; then
    return 0
  fi
  if command -v brew >/dev/null 2>&1; then
    info "Installing Rust with Homebrew"
    brew install rust
    if ! command -v cargo >/dev/null 2>&1; then
      eval "$(brew shellenv)"
    fi
  else
    info "Installing Rust with rustup"
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # shellcheck disable=SC1091
    . "${HOME}/.cargo/env"
  fi
  command -v cargo >/dev/null 2>&1 || die "cargo is not on PATH after installing Rust"
}

local_source_dir() {
  local source_path="${BASH_SOURCE[0]:-}"
  if [ -z "$source_path" ] || [ ! -f "$source_path" ]; then
    return 0
  fi
  local dir
  dir="$(cd "$(dirname "$source_path")" && pwd)"
  if [ -f "${dir}/Cargo.toml" ]; then
    printf '%s\n' "$dir"
  fi
}

install_with_brew() {
  info "Installing git-whistles with Homebrew"
  brew tap mezis/git-whistles "$GIT_WHISTLES_GIT_URL" || return 1
  # Reinstall picks up a formula that switched from a source HEAD build to
  # the published release tarball.
  if brew list --formula git-whistles >/dev/null 2>&1; then
    brew reinstall mezis/git-whistles/git-whistles || return 1
  else
    brew install mezis/git-whistles/git-whistles || return 1
  fi
}

install_with_cargo() {
  ensure_cargo
  local -a args
  args=(install --locked --force)
  local source_dir
  source_dir="$(local_source_dir || true)"
  if [ -n "$source_dir" ]; then
    info "Building git-whistles from ${source_dir}"
    args+=(--path "$source_dir")
  else
    info "Building git-whistles from ${GIT_WHISTLES_GIT_URL} (${GIT_WHISTLES_GIT_BRANCH})"
    args+=(--git "$GIT_WHISTLES_GIT_URL" --branch "$GIT_WHISTLES_GIT_BRANCH")
  fi
  cargo "${args[@]}"
  if [ -f "${HOME}/.cargo/env" ]; then
    # shellcheck disable=SC1091
    . "${HOME}/.cargo/env"
  fi
}

install_binary() {
  export HOMEBREW_NO_AUTO_UPDATE=1
  local source_dir=""
  source_dir="$(local_source_dir || true)"
  case "$GIT_WHISTLES_INSTALL_FROM" in
    brew)
      install_with_brew
      ;;
    cargo)
      install_with_cargo
      ;;
    auto)
      # A checkout next to this script is the tree to build. curl | bash has
      # no script path, so prefer Homebrew and fall back to cargo from GitHub.
      if [ -n "$source_dir" ]; then
        install_with_cargo
      elif command -v brew >/dev/null 2>&1; then
        if ! install_with_brew; then
          info "Homebrew install failed; building with cargo instead"
          install_with_cargo
        fi
      else
        install_with_cargo
      fi
      ;;
    *)
      die "GIT_WHISTLES_INSTALL_FROM must be auto, brew, or cargo"
      ;;
  esac
  hash -r || true
}

# Point shims at ~/.local/bin/git-whistles rather than a Homebrew cellar path.
# current_exe() inside `git-whistles shim` resolves symlinks, so a later
# `brew upgrade` would otherwise leave the shims aimed at an old cellar build.
retarget_our_shims() {
  local shim_dir="$1"
  local real_binary="$2"
  local stable="$3"
  local link target
  shopt -s nullglob
  for link in "${shim_dir}"/git-*; do
    [ "$link" = "$stable" ] && continue
    [ -L "$link" ] || continue
    target="$(readlink "$link")"
    if [ "$target" = "$stable" ]; then
      continue
    fi
    if [ "$(resolve_path "$link")" = "$real_binary" ]; then
      ln -sfn "$stable" "$link"
    fi
  done
  shopt -u nullglob
}

publish_on_path() {
  mkdir -p "$SHIM_DIR"
  local installed resolved stable_resolved
  installed="$(command -v git-whistles || true)"
  [ -n "$installed" ] || die "git-whistles is not on PATH after install"
  resolved="$(resolve_path "$installed")"
  stable_resolved=""
  if [ -e "$STABLE_BIN" ]; then
    stable_resolved="$(resolve_path "$STABLE_BIN")"
  fi
  if [ "$resolved" != "$stable_resolved" ]; then
    ln -sfn "$installed" "$STABLE_BIN"
  fi
  "$installed" shim --dir "$SHIM_DIR"
  retarget_our_shims "$SHIM_DIR" "$resolved" "$STABLE_BIN"
}

write_env_file() {
  mkdir -p "$(dirname "$ENV_FILE")" "$SHIM_DIR"
  local tmp
  tmp="$(mktemp)"
  cat >"$tmp" <<'EOF'
# git-whistles: PATH entries and the wt alias. Safe to source more than once.
if [ -d "${HOME}/.cargo/bin" ]; then
  case ":${PATH}:" in
    *":${HOME}/.cargo/bin:"*) ;;
    *) PATH="${HOME}/.cargo/bin:${PATH}" ;;
  esac
fi
if [ -d "${HOME}/.local/bin" ]; then
  case ":${PATH}:" in
    *":${HOME}/.local/bin:"*) ;;
    *) PATH="${HOME}/.local/bin:${PATH}" ;;
  esac
fi
export PATH
alias wt='eval "$(git-whistles worktree-list)"'
EOF
  mv "$tmp" "$ENV_FILE"
}

ensure_rc_sources_env() {
  local rc="$1"
  if [ -f "$rc" ] && grep -qF "$MARKER_BEGIN" "$rc"; then
    return 0
  fi
  if [ -s "$rc" ] && [ "$(tail -c1 "$rc" | wc -l | tr -d ' ')" -eq 0 ]; then
    printf '\n' >>"$rc"
  fi
  cat >>"$rc" <<EOF

${MARKER_BEGIN}
[ -f "\$HOME/.config/git-whistles/env.sh" ] && . "\$HOME/.config/git-whistles/env.sh"
${MARKER_END}
EOF
}

install_shell_integration() {
  write_env_file
  ensure_rc_sources_env "${HOME}/.zshrc"
  ensure_rc_sources_env "${HOME}/.bashrc"
  # macOS bash is a login shell and reads .bash_profile, not .bashrc.
  # Only touch .bash_profile when it already exists: creating one would hide ~/.profile.
  if [ -f "${HOME}/.bash_profile" ]; then
    ensure_rc_sources_env "${HOME}/.bash_profile"
  fi
}

print_done() {
  cat <<'EOF' >&2

git-whistles is installed.
Restart your shell (or: source ~/.config/git-whistles/env.sh), then:

  wt                   pick a worktree and cd into it
  git worktree-list    same picker, via the shim
  git-whistles --help

EOF
}

main() {
  assert_supported_os "$(uname -s)"
  locate_brew || true
  ensure_git
  install_binary
  publish_on_path
  install_shell_integration
  print_done
}

if [ "${GIT_WHISTLES_INSTALL_LIB:-}" != 1 ]; then
  main "$@"
fi
