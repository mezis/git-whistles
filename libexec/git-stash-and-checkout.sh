#!/bin/bash
#
# git-stash-and-checkout --
# 
# Push to the stash, checkout, and pop the relevant WIP stash.
#

die() {
  echo "$@" ; exit 1
}

head=$(git symbolic-ref HEAD 2> /dev/null || git log -1 --format=%h)
current_branch=${head#refs/heads/}
target_branch=$1

git stash --include-untracked || die
git checkout $target_branch || die

stash=$(git stash list | grep "WIP on ${target_branch}:" | head -1)
stash=${stash%%:*}

if test -n "$stash" ; then
  echo "Popping $stash"
  git stash pop $stash
fi
