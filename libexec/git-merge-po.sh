#!/bin/sh
#
# git-merge-po --
# 
#   Custom Git merge driver - merges PO files using msgcat(1)
#
# - Add this to your .git/config :
#
#   [merge "pofile"]
#     name = Gettext merge driver
#     driver = git merge-po %O %A %B
# 
# - Add this to .gitattributes :
# 
#   *.po   merge=pofile
#   *.pot  merge=pofile
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
# 
O=$1
A=$2
B=$3

# Extract the PO header from the current branch (top of file until first empty line)
header=$(mktemp /tmp/merge-po.XXXX)
sed -e '/^$/q' < $A > $header

# Merge files, then repair header
temp=$(mktemp /tmp/merge-po.XXXX)
msgcat -o $temp $A $B
msgcat --use-first -o $A $header $temp

# Clean up
rm $header $temp

# Check for conflicts
conflicts=$(grep -c "#-#" $A)
test $conflicts -gt 0 && exit 1
exit 0
