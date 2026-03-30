# git-whistles

Helpers for classic [Git](https://git-scm.com/) workflows. Written in Rust.

## Install

**From GitHub Releases (recommended)**

Download the tarball for your platform from [Releases](https://github.com/mezis/git-whistles/releases) and extract `git-whistles` into your `PATH` (e.g. `~/bin` or `/usr/local/bin`).

**Homebrew (tap from this repo)**

```bash
brew tap mezis/git-whistles
brew install git-whistles
```

Upgrade: `brew update && brew upgrade git-whistles`

**From source (Cargo)**

```bash
cargo install --path .
# or from crates.io once published:
# cargo install git-whistles
```

**Optional: install shims**

After installing the binary, you can add symlinks so each Git-style command is available as `git-<subcommand>` (e.g. `git-chop`, `git-merge-po`). `shim` and `unshim` are not shimmed; run them as `git-whistles shim` / `git-whistles unshim`.

```bash
git-whistles shim
# Default target is /usr/local/bin. Use another directory:
git-whistles shim --dir ~/bin
```

Remove shims: `git-whistles unshim [--dir /usr/local/bin]`

## Debugging

Pass **`-x`** or **`--echo-commands`** on any invocation (before or after the subcommand) to print each external command to stderr before it runs, similar to `set -x`:

```bash
git-whistles -x chop my-branch
git staging -x
```

## Commands

- **`git chop [branch1 ...]`** — Delete local and remote branch(es). If you're on a branch being chopped, checks out the primary branch (main/master) first.

- **`git ff-all-branches [--no-fetch] [-p] [-v] [-q] [-r REMOTE]`** — Fast-forward all local tracking branches to their remote counterpart where possible. Fetches first by default; `--no-fetch` skips that step; `-p` dry-run; `-v` verbose.

- **`git list-branches [-l] [-r] [-i integration-branch] [-p]`** — List local or remote branches and their distance to an integration branch (default `origin/master`). `-p` porcelain (CSV).

- **`git stash-and-checkout <branch>`** — Stash (including untracked), checkout the branch, then pop the matching WIP stash if any.

- **`git staging [branch]`** — Sync the given branch (or current) with main: stash-and-checkout → ff-all-branches → merge main → push → stash, checkout staging → fetch, reset --hard origin/staging → merge branch → push → stash-and-checkout back. Use when you want to land a feature branch into a `staging` branch.

- **`git merge-po <base> <local> <remote>`** — Three-way merge driver for gettext PO files. Uses `msguniq`, `msgcat`, `msgmerge`, `msggrep`. Not meant to be run by hand; use as a merge driver (see below).

- **`git changes`** — Show commits on the current branch that are not in the primary remote branch. The primary branch is detected from `origin/HEAD`, or `origin/main`, or `origin/master`.

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
