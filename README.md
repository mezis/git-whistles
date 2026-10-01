# git-whistles

Helpers for classic [Git](https://git-scm.com/) workflows. Written in Rust.

## Install

macOS and Linux. Restart your shell afterwards.

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/mezis/git-whistles/master/install.sh | bash
```

That builds git-whistles from source, puts `git-<command>` shims on your `PATH`, and defines:

```bash
alias wt='eval "$(git-whistles worktree-list)"'
```

Running it again is safe. When `brew` is available it installs with Homebrew (`brew install --HEAD`, which compiles this repo). Otherwise it installs Rust if needed (Homebrew, or rustup) and runs `cargo install`.

**Manual build**

```bash
cargo install --path . --locked
git-whistles shim --dir ~/.local/bin
```

`shim` and `unshim` are not shimmed; run them as `git-whistles shim` / `git-whistles unshim`. Remove shims with `git-whistles unshim --dir ~/.local/bin`.

## Debugging

Pass **`-x`** or **`--echo-commands`** on any invocation (before or after the subcommand) to print each external command to stderr before it runs, similar to `set -x`.

Pass **`-v`** or **`--stream-commands`** to stream subprocess output to your terminal when git-whistles is not capturing that output for parsing (for commands that only capture stdout, stderr is still streamed).

```bash
git-whistles -x chop my-branch
git staging -x
git-changes -v
```

## Commands

- **`git chop [branch1 ...]`** — Delete local and remote branch(es). If you're on a branch being chopped, checks out the primary branch (main/master) first.

- **`git ff-all-branches [--no-fetch] [-p] [--progress] [-q] [-r REMOTE]`** — Fast-forward all local tracking branches to their remote counterpart where possible. Fetches first by default; `--no-fetch` skips that step; `-p` dry-run; **`--progress`** prints a line for each branch that is or would be updated. Global **`-v` / `--stream-commands`** streams underlying `git` subprocess output (see Debugging).

- **`git list-branches [-l] [-r] [-i integration-branch] [-p]`** — List local or remote branches and their distance to an integration branch (default: same primary as `git changes`: `origin/HEAD` if set, else `origin/main` or `origin/master`). `-p` porcelain (CSV).

- **`git worktree-list`** / **`wt`** — Interactive table of worktrees from every main clone this command has been run in (repo, branch, path; long paths elide the prefix). Arrow keys move; typing letters/digits filters (subsequence match). **Enter** prints `cd -- 'path'` (`wt` is `eval "$(git-whistles worktree-list)"`; quotes required). **Esc** prints nothing. **Ctrl-d** destroys a linked worktree after confirm (`docker compose down` and/or `bin/teardown` when present, then `git worktree remove`). Cannot destroy the main checkout or the worktree the shell is in.

- **`git stash-and-checkout <branch>`** — Stash (including untracked), checkout the branch, then pop the matching WIP stash if any.

- **`git staging [branch]`** — Sync the given branch (or current) with main: stash-and-checkout → ff-all-branches → merge main → push → stash, checkout staging → fetch, reset --hard origin/staging → merge branch → push → stash-and-checkout back. Use when you want to land a feature branch into a `staging` branch.

- **`git merge-po <base> <local> <remote>`** — Three-way merge driver for gettext PO files. Uses `msguniq`, `msgcat`, `msgmerge`, `msggrep`. Not meant to be run by hand; use as a merge driver (see below).

- **`git changes [git log args…]`** — Show commits on the current branch that are not in the primary remote branch. The primary branch is detected from `origin/HEAD`, or `origin/main`, or `origin/master`. Extra arguments are passed through to `git log` (for example `git changes --stat` or `git changes -- path/to/file`). Global `-v` / `-x` stay git-whistles flags; use `--` if you need to pass those to `git log`.

- **`git-whistles shim [--dir DIR]`** / **`git-whistles unshim [--dir DIR]`** — Add or remove `git-<subcommand>` symlinks to the main binary (not `shim` / `unshim` themselves). Default dir: `/usr/local/bin`.

You can run the binary as `git-whistles <subcommand>` or install shims and run e.g. `git-chop` or `git merge-po` (after `git-chop` / `git-merge-po` are on `PATH`).

## merge-po setup

Use as a Git merge driver for `.po` / `.pot` files.

**Repo-local** — in `.git/config`:

```ini
[merge "pofile"]
  name = Gettext merge driver
  driver = git merge-po %O %A %B
```

In `.gitattributes`:

```
*.po   merge=pofile
*.pot  merge=pofile
```

**Global** — in `~/.gitconfig`:

```ini
[core]
  attributesfile = ~/.gitattributes
[merge "pofile"]
  name = Gettext merge driver
  driver = git merge-po %O %A %B
```

And in `~/.gitattributes`:

```
*.po   merge=pofile
*.pot  merge=pofile
```

Requires gettext (`msguniq`, `msgcat`, `msgmerge`, `msggrep`) on `PATH`.

## Build and test

Gettext (`msguniq`, `msgcat`, `msgmerge`, `msggrep`) is **mandatory** for the full test suite. Install it first (e.g. `apt-get install gettext`, `brew install gettext`).

```bash
cargo build --release
cargo test
```

## License

MIT.
