git whistles [![Gem Version](https://badge.fury.io/rb/git-whistles.png)](http://badge.fury.io/rb/git-whistles)
=====

This repository is about sharing helper scripts for the [Git](http://git-scm.com/) version control system.

Install with:

    gem install git-whistles

### Commands available:

`git chop [branch1, ...]` - Deletes the local and origin copy of a branch. Useful to close feature branches once a feature is completed. It also accepts multiple branches separated by spaces

`git pr [--from your-branch] [--to target-branch]` - Open your browser at a Github pull-request page for the specified branch (defaults to the current `head`).

`git explore [-r REF] [-p PATH]` - Opens the remote origin interface on the given reference and path. Reference defaults to current branch and path to root

`git ff-all-branches [-f] [-p] [-v]` - Fast-forward all local tracking branches to their remote counterpart (where possible). Very useful on big projects.

`git jira-branch <issue-id>` - Creates a branch name suggestion from the specified JIRA issue ID. It also writes the branch name as a comment on the issue and allows to transition the issue.

`git jira-pr [--from your-branch] [--to target-branch]` - Open your browser at a Github pull-request page for the specified branch (defaults to the current `head`). If you're using JIRA and your branch has a issue id in its name, populates your pull request with that issue details, else it just creates an empty Pull Request from the current HEAD.

`git latest-pushes [-n NR_RESULTS] [-p PATTERN]` - Show latest pushed branches to origin. Defaults to 20 results. Pattern is appended to refs/remotes/origin/ so include the team or project name to filter results.

`git list-branches [-l] [-r] [-i integration-branch]` - Colourful listing of all local or origin branches, and their distance to an integration branch (`master` by default).

`git merge-po <ancestor> <left> <right>` - Merge engine for GetText PO files.

`git outstanding-features [-f from-branch] [-t to-branch] [--oneline]` - List the pull requests merged in `[to-branch]` but not in `[from-branch]`. Useful to prepare a list of stuff you're going to deploy. Defaults to listing what's on `origin/master` but not on `origin/production`. By default lists one feature per line, if `--oneline` or `-o` is specified features will be separated by spaces instead.

`git pivotal-branch <story-id>` - Creates a branch name suggestion from the specified Pivotal Tracker story ID. It also starts the story and writes the branch name as a comment.

`git pivotal-open <story-id>` - Opens the Pivotal Tracker story page for the current branch, from the specified Pivotal Tracker story ID or it is inferred from the branch name if not supplied

`git pivotal-pr [--from your-branch] [--to target-branch]` - Open your browser at a Github pull-request page for the specified branch (defaults to the current `head`). If you're using Pivotal Tracker and your branch has a story number in its name, populates your pull request with story details, else it just creates an empty Pull Request from the current HEAD.

`git select <story-id> [-p PREFIX] [-r REMOTE_REF]` - Checkout a local branch with the matching number. If not found, lists remote branches. With `-p` you can add a prefix to your local branch name. The `-r` options allows to checkout a branch in remote if you don't have a local branch with that number.

`git stash-and-checkout [branch]` - Stash and checkout another branch.

`git youtrack-branch <ticket-id>` - Creates a branch name suggestion from the specified Youtrack ticket ID.

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

Or, if you want to make this setting global:

Create a user global file ~/.gitattributes and fill it with:

    *.po   merge=pofile
    *.pot  merge=pofile

Add this to your user global ~/.gitconfig:

    [core]
      attributesfile = ~/.gitattributes
    [merge "pofile"]
      name = Gettext merge driver
      driver = git merge-po %O %A %B

### JIRA

All JIRA commands require a JIRA username, [API token](https://id.atlassian.com/manage/api-tokens) and site. Please use the following commands
to set it up:

```
$ git config [--global] jira.username <username>
$ git config [--global] jira.token <token>
$ git config [--global] jira.site <https://mydomain.atlassian.net>
```

### Pivotal

All pivotal commands require a Pivotal Tracker token. The token needs to be generated on
the Pivotal Tracker UI. The token can then be set locally via the following command:

```
$ git config [--global] pivotal-tracker.token <token>
```

### Youtrack

All Youtrack commands require a Youtrack username, password and url. Please use the following commands to set it up:

```
$ git config [--global] youtrack.username <username>
$ git config [--global] youtrack.password <password>
$ git config [--global] youtrack.url <https://your_youtrack_url.com>
```

### License

Released on the MIT license terms.
