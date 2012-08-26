#!/bin/sh
#
# git-outstanding-features --
# 
# List merged pull requests.
#

from=${1:-origin/production}
to=${2:-production}

git log --oneline ${to} ^${from} | grep 'Merge pull request' | sed  -e 's:.*from [^/]*/::'
