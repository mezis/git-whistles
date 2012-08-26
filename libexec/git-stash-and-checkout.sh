#!/bin/bash
#
# git-stash-and-checkout --
# 
# Push to the stash, checkout, and pop the relevant WIP stash.
#
# Copyright (C) 2012 Julien Letessier
# 
# Permission is hereby granted, free of charge, to any person obtaining a copy of
# this software and associated documentation files (the "Software"), to deal in
# the Software without restriction, including without limitation the rights to
# use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
# of the Software, and to permit persons to whom the Software is furnished to do
# so, subject to the following conditions:
# 
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
# 
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#

die() {
  echo "$@" ; exit 1
}

head=$(git symbolic-ref HEAD 2> /dev/null || git log -1 --format=%h)
current_branch=${head#refs/heads/}
target_branch=$1

git stash --include-untracked || die
git checkout $target_branch || die

stash=$(git stash list | grep "WIP on ${target_branch}:")
stash=${stash%%:*}

if test -n "$stash" ; then
  echo "Popping $stash"
  git stash pop $stash
fi
