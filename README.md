
This repository is about sharing helper scripts for the [Git](http://git-scm.com/)
version control system.

Install with:

`gem install git-whistles`

## chop

`git chop [branch]`

Delete the local and origin copy of a branch.
Useful to close feature branches once a feature is completed.

## list-branches

`git list-branches [-l] [-r] [-i integration-branch]`

Colourful listing of all local or origin branches, and their distance to an
integration branch (`master` by default).

## merge-po

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


## pull-request

`git-pull-request [branch]`

Open your browser at a Github pull-request page for the specified branch
(defaults to the current `head`).


## stash-and-checkout

`git stash-and-checkout [branch]`

As the name implies: stash and checkout another branch.
If there was work in progress previously stashes for the target branch, it gets
unstashed.

This lets you keep work in progress on multiple branches without committing it.

I tend to alias this to `git co`.
