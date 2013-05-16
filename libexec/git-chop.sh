#!/bin/bash
#
# git-chop --
# 
#  Close a feature branch - removing it from local and remote.
#

die() {
  echo "$@" ; exit 1
}

head=$(git symbolic-ref HEAD 2> /dev/null || git log -1 --format=%h)
current_branch=${head#refs/heads/}

branch="$1"
: ${branch:-$head}
echo "Closing feature branch $branch"

if [ "$branch" = "$current_branch" ] ; then
  git checkout master || die
fi

git branch -D "$branch" || die
git push origin ":$branch" || die
