#!/usr/bin/env bash
# Exercises install.sh shell setup without building or touching the real HOME.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
home_dir="$(mktemp -d)"
trap 'rm -rf "$home_dir"' EXIT

export HOME="$home_dir"
export GIT_WHISTLES_INSTALL_LIB=1
# shellcheck disable=SC1091
source "${repo_root}/install.sh"

assert_supported_os Linux
assert_supported_os Darwin
set +e
(assert_supported_os Windows)
status=$?
set -e
test "$status" -ne 0

install_shell_integration
install_shell_integration

begin_count="$(grep -cF '# >>> git-whistles initialize >>>' "${HOME}/.zshrc")"
end_count="$(grep -cF '# <<< git-whistles initialize <<<' "${HOME}/.bashrc")"
test "$begin_count" -eq 1
test "$end_count" -eq 1
test ! -f "${HOME}/.bash_profile"

touch "${HOME}/.bash_profile"
install_shell_integration
test "$(grep -cF '# >>> git-whistles initialize >>>' "${HOME}/.bash_profile")" -eq 1

grep -F "alias wt='eval \"\$(git-whistles worktree-list)\"'" "${HOME}/.config/git-whistles/env.sh" >/dev/null

export PATH="/usr/bin"
# shellcheck disable=SC1091
. "${HOME}/.config/git-whistles/env.sh"
# shellcheck disable=SC1091
. "${HOME}/.config/git-whistles/env.sh"
case ":${PATH}:" in
  *":${HOME}/.local/bin:"*) ;;
  *)
    printf 'local bin missing from PATH: %s\n' "$PATH" >&2
    exit 1
    ;;
esac
path_count="$(printf '%s' "$PATH" | awk -F: -v dir="${HOME}/.local/bin" '{c=0; for(i=1;i<=NF;i++) if($i==dir) c++; print c}')"
test "$path_count" -eq 1

# Shims that resolve to our binary are retargeted. Other git-* symlinks stay.
bin_dir="${HOME}/fake-bin"
shim_dir="${HOME}/shims"
mkdir -p "$bin_dir" "$shim_dir"
printf '#!/bin/sh\n' >"${bin_dir}/git-whistles"
chmod +x "${bin_dir}/git-whistles"
real_binary="$(resolve_path "${bin_dir}/git-whistles")"
stable="${shim_dir}/git-whistles"
ln -s "${bin_dir}/git-whistles" "$stable"
ln -s "$real_binary" "${shim_dir}/git-chop"
ln -s /bin/ls "${shim_dir}/git-lfs"
retarget_our_shims "$shim_dir" "$real_binary" "$stable"
test "$(readlink "${shim_dir}/git-chop")" = "$stable"
test "$(readlink "${shim_dir}/git-lfs")" = /bin/ls
test "$(readlink "$stable")" = "${bin_dir}/git-whistles"

printf 'install.sh shell setup ok\n'
