git whistles [![Gem Version](https://badge.fury.io/rb/git-whistles.png)](http://badge.fury.io/rb/git-whistles)
=====

This repository is about sharing helper scripts for the [Git](http://git-scm.com/) version control system.

Install with:

    gem install git-whistles

Use it with:


| Command | Description|
|---------|------------|
| `git ff-all-branches [-f] [-p] [-v]`  | Fast-forward all local tracking branches to their remote counterpart (where possible). Very useful on big projects. |
| `git stash-and-checkout [branch]`     | As the name implies: stash and checkout another branch. |
| `git pull-request [--from your-branch] [--to target-branch]` | Open your browser at a Github pull-request page for the specified branch (defaults to the current `head`). If you're using Pivotal Tracker and your branch has a story number in its name, populates your pull request with story details. |
| `git outstanding-features [-f from-branch] [-t to-branch] [--oneline]` | List the pull requests merged in `[to-branch]` but not in `[from-branch]`. Useful to prepare a list of stuff you're going to deploy. Defaults to listing what's on `origin/master` but not on `origin/production`. By default lists one feature per line, if `--oneline` or `-o` is specified features will be separated by spaces instead. [[PedroCunha](https://github.com/PedroCunha)] |
| `git chop [branch ...]` | Deletes the local and origin copy of a branch. Useful to close feature branches once a feature is completed. It also accepts multiple branches separated by spaces [[David Silva](https://github.com/Davidslv)] |
| `git list-branches [-l] [-r] [-i integration-branch]` | Colourful listing of all local or origin branches, and their distance to an integration branch (`master` by default). |
| `git merge-po <ancestor> <left> <right>` | Merge engine for GetText PO files. |
| `git select <story-id>`                  | Checkout a local branch with the matching number. If not found, lists remote branches |
| `git latest-pushes [-n NR_RESULTS] [-p PATTERN]` | Show latest pushed branches to origin. Defaults to 20 results. Pattern is appended to refs/remotes/origin/ so include the team or project name to filter results. [[PedroCunha](https://github.com/PedroCunha)] |
| `git pivotal-branch <story-id>` | Creates a branch name suggestion from the specified Pivotal Tracker story ID. It also comments on the story the branch name created and starts the story [[dncrht](https://github.com/dncrht)] |
| `git pivotal-open [story-id]` | Opens the Pivotal Tracker story page for the current branch, from the specified Pivotal Tracker story ID or it is inferred from the branch name if not supplied [[khiet](https://github.com/khiet)] |
| `git explore [-r REF] [-p PATH]` | Opens the remote origin interface on the given reference and path. Reference defaults to current branch and path to root [[PedroCunha](https://github.com/pedrocunha)]|


### More details on some of the commands


#### merge-po

`git merge-po <ancestor> <left> <right>`

For those using `gettext` for I18n, a must-have: this custom merge driver 
will handle most merge/conflicts issues when a PO file was edited by different
committers.

You don't have to call this directly.

Add this to your .git/config:

    [merge "pofile"]
      name = Gettext merge driver
      driver = git merge-po %O %A %B

Add this to .gitattributes:

    *.po   merge=pofile
    *.pot  merge=pofile


#### stash-and-checkout

`git stash-and-checkout [branch]`

As the name implies: stash and checkout another branch.
If there was work in progress previously stashes for the target branch, it gets
unstashed.

This lets you keep work in progress on multiple branches without committing it.

I tend to alias this to `git co`.


### License

Released on the MIT license terms.
