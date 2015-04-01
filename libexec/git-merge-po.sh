#!/bin/bash
# git merge driver for .PO files
# Copyright (c) Mikko Rantalainen <mikko.rantalainen@peda.net>, 2013
# License: MIT
# From: http://stackoverflow.com/questions/16214067/wheres-the-3-way-git-merge-driver-for-po-gettext-files

ORIG_HASH=$(git hash-object "${1}")
WORKFILE=$(git ls-tree -r HEAD | fgrep "$ORIG_HASH" | cut -b54-)
echo "Using custom merge driver for $WORKFILE..."

BASE="${1}._BASE_"
LOCAL="${2}._LOCAL_"
REMOTE="${3}._REMOTE_"

LOCAL_ONELINE="$LOCAL""ONELINE_"
BASE_ONELINE="$BASE""ONELINE_"
REMOTE_ONELINE="$REMOTE""ONELINE_"

OUTPUT="$LOCAL""OUTPUT_"
MERGED="$LOCAL""MERGED_"
MERGED2="$LOCAL""MERGED2_"

TEMPLATE1="$LOCAL""TEMPLATE1_"
TEMPLATE2="$LOCAL""TEMPLATE2_"
FALLBACK_OBSOLETE="$LOCAL""FALLBACK_OBSOLETE_"

# standardize the input files for regexping
# default to UTF-8 in case charset is still the placeholder "CHARSET"
cat "${1}" | perl -npe 's!(^"Content-Type: text/plain; charset=)(CHARSET)(\\n"$)!$1UTF-8$3!' | msgcat --no-wrap --sort-output - > "$LOCAL"
cat "${2}" | perl -npe 's!(^"Content-Type: text/plain; charset=)(CHARSET)(\\n"$)!$1UTF-8$3!' | msgcat --no-wrap --sort-output - > "$BASE"
cat "${3}" | perl -npe 's!(^"Content-Type: text/plain; charset=)(CHARSET)(\\n"$)!$1UTF-8$3!' | msgcat --no-wrap --sort-output - > "$REMOTE"

# convert each definition to single line presentation
# extra fill is required to make sure that git separates each conflict 
perl -npe 'BEGIN {$/ = "\n\n"}; s/#\n$/\n/s; s/#/##/sg; s/\n/#n/sg; s/#n$/\n/sg; s/#n$/\n/sg; $_.="#fill#\n" x 4' "$LOCAL" > "$LOCAL_ONELINE"
perl -npe 'BEGIN {$/ = "\n\n"}; s/#\n$/\n/s; s/#/##/sg; s/\n/#n/sg; s/#n$/\n/sg; s/#n$/\n/sg; $_.="#fill#\n" x 4' "$BASE"  > "$BASE_ONELINE"
perl -npe 'BEGIN {$/ = "\n\n"}; s/#\n$/\n/s; s/#/##/sg; s/\n/#n/sg; s/#n$/\n/sg; s/#n$/\n/sg; $_.="#fill#\n" x 4' "$REMOTE"  > "$REMOTE_ONELINE"

# merge files using normal git merge machinery
git merge-file -p --union -L "Current (working directory)" -L "Base (common ancestor)" -L "Incoming (applied changeset)" "$LOCAL_ONELINE" "$BASE_ONELINE" "$REMOTE_ONELINE" > "$MERGED"
MERGESTATUS=$?

# remove possibly duplicated headers (workaround msguniq bug http://comments.gmane.org/gmane.comp.gnu.gettext.bugs/96)
cat "$MERGED" | perl -npe 'BEGIN {$/ = "\n\n"}; s/^([^\n]+#nmsgid ""#nmsgstr ""#n.*?\n)([^\n]+#nmsgid ""#nmsgstr ""#n.*?\n)+/$1/gs' > "$MERGED2"

# remove lines that have totally empty msgstr
# and convert back to normal PO file representation
cat "$MERGED2" | grep -v '#nmsgstr ""$' | grep -v '^#fill#$' | perl -npe 's/#n/\n/g; s/##/#/g' > "$MERGED"

# run the output through msguniq to merge conflicts gettext style
# msguniq seems to have a bug that causes empty output if zero msgids
# are found after the header. Expected output would be the header...
# Workaround the bug by adding an empty obsolete fallback msgid
# that will be automatically removed by msguniq

cat > "$FALLBACK_OBSOLETE" << 'EOF'

#~ msgid "obsolete fallback"
#~ msgstr ""

EOF
cat "$MERGED" "$FALLBACK_OBSOLETE" | msguniq --no-wrap --sort-output > "$MERGED2"


# create a hacked template from default merge between 3 versions
# we do this to try to preserve original file ordering
msgcat --use-first "$LOCAL" "$REMOTE" "$BASE" > "$TEMPLATE1"
msghack --empty "$TEMPLATE1" > "$TEMPLATE2"
msgmerge --silent --no-wrap --no-fuzzy-matching "$MERGED2" "$TEMPLATE2" > "$OUTPUT"

# show some results to stdout
if grep -q '#-#-#-#-#' "$OUTPUT"
then
    FUZZY=$(cat "$OUTPUT" | msgattrib --only-fuzzy --no-obsolete --color | perl -npe 'BEGIN{ undef $/; }; s/^.*?msgid "".*?\n\n//s')
    if test -n "$FUZZY"
    then
        echo "-------------------------------"
        echo "Fuzzy translations after merge:"
        echo "-------------------------------"
        echo "$FUZZY"
        echo "-------------------------------"
    fi
fi

# git merge driver must overwrite the first parameter with output
mv "$OUTPUT" "${1}"

# cleanup
rm -f "$LOCAL" "$BASE" "$REMOTE" "$LOCAL_ONELINE" "$BASE_ONELINE" "$REMOTE_ONELINE" "$MERGED" "$MERGED2" "$TEMPLATE1" "$TEMPLATE2" "$FALLBACK_OBSOLETE"

# return conflict if merge has conflicts according to msgcat/msguniq
grep -q '#-#-#-#-#' "${1}" && exit 1

# otherwise, return git merge status
exit $MERGESTATUS

# Steps to install this driver:
# (1) Edit ".git/config" in your repository directory
# (2) Add following section:
#
# [merge "merge-po-files"]
#   name = merge po-files driver
#   driver = ./bin/merge-po-files %A %O %B
#   recursive = binary
#
# or
#
# git config merge.merge-po-files.driver "./bin/merge-po-files %A %O %B"
#
# The file ".gitattributes" will point git to use this merge driver.
